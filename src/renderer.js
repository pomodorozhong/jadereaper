// WebGPU renderer: one background pass and one instanced draw of paper parts.
import { t } from './i18n.js';
import { PART_IDS } from './data.js';
import { BAKE, bakeCharacters } from './characters.js';

const INSTANCE_BYTES = 48;
const MAX_INSTANCES = 4096;
const WORLD_HEIGHT = 640; // world units across the shorter screen side
// The core sends part id + PART_SLOTS * pose (core/src/defs.rs), and the rect table has a row for each.
const PART_SLOTS = 32;
const MAX_POSES = 8;

/** Look settings sent to the shader each frame. The ?debug panel changes them at run time. */
// Tuned by the maintainer in the ?debug panel. The line boil replaces the old outline wobble, which stays off.
export const LOOK = {
  stopMotionFps: 0,   // how often the old outline wobble steps (off)
  boilFps: 4,         // how often the traced characters change pose; the core picks each instance's pose
  margin: 2.1,        // white cutout margin, world units
  shadowX: 3.5, shadowY: 6,
  shadowAlpha: 0.5,
  wobble: 0,          // the shader's old outline wobble on cutout parts, world units (off)
};

/**
 * Pack the part images into one atlas canvas: optional images listed in assets/parts.json, and the hand-traced
 * characters mapped in assets/characters.json, baked in bake.poses poses. Returns the rect table: row
 * id + PART_SLOTS * pose holds that pose's image, and a part with one image has it in every pose's row.
 */
async function loadPartImages(bake) {
  const rects = new Float32Array(PART_SLOTS * MAX_POSES * 4);
  const setRect = (id, pose, rect) => rects.set(rect, (id + PART_SLOTS * pose) * 4);
  let manifest = {};
  try {
    const res = await fetch('assets/parts.json', { cache: 'no-cache' });
    if (res.ok) manifest = await res.json();
  } catch {
    // No manifest: every part is drawn in code.
  }
  const entries = Object.entries(manifest).filter(([name]) => name in PART_IDS);
  const [images, baked] = await Promise.all([
    Promise.all(entries.map(async ([name, path]) => {
      try {
        const blob = await (await fetch(`assets/${path}`)).blob();
        return { id: PART_IDS[name], bmp: await createImageBitmap(blob) };
      } catch (err) {
        console.warn(`Could not load part image for "${name}"`, err);
        return null;
      }
    })),
    bakeCharacters(PART_IDS, bake),
  ]);
  if (!images.some(Boolean) && !baked.length) return { rects, canvas: null };

  // 2048 holds the default bake; sharper or more poses (from the debug panel) need 4096.
  let area = 0;
  for (const { poses } of baked) for (const c of poses || []) area += (c.width + 4) * (c.height + 4);
  const SIZE = area > 2048 * 2048 * 0.7 ? 4096 : 2048;
  // When even 4096 is too small (big sizes from the debug panel), draw every baked image smaller so all fit.
  const fit = Math.min(1, Math.sqrt((SIZE * SIZE * 0.7) / (area || 1)));
  const GAP = 4;
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = SIZE;
  const ctx = canvas.getContext('2d');
  let x = GAP, y = GAP, rowH = 0;
  /** Place one image; returns its rect, or null when the atlas is full. */
  const place = (bmp, shrink = 1) => {
    // Shrink anything larger than a quarter of the atlas's side.
    const scale = Math.min(shrink, SIZE / 4 / Math.max(bmp.width, bmp.height));
    const w = Math.round(bmp.width * scale), h = Math.round(bmp.height * scale);
    if (x + w + GAP > SIZE) { x = GAP; y += rowH + GAP; rowH = 0; }
    if (y + h + GAP > SIZE) { console.warn('Part atlas is full, skipping an image'); return null; }
    ctx.drawImage(bmp, x, y, w, h);
    const rect = [x / SIZE, y / SIZE, w / SIZE, h / SIZE];
    x += w + GAP;
    rowH = Math.max(rowH, h);
    return rect;
  };
  for (const img of images) {
    const rect = img && place(img.bmp);
    if (rect) for (let k = 0; k < MAX_POSES; k++) setRect(img.id, k, rect);
  }
  // A hidden part points at the atlas's last pixel, which nothing draws on, so it samples as clear.
  const clear = [(SIZE - 1) / SIZE, (SIZE - 1) / SIZE, 0.5 / SIZE, 0.5 / SIZE];
  for (const { id, poses } of baked) {
    const placed = poses ? poses.map((c) => place(c, fit)) : [clear];
    // Fill every pose row, so a core that has not caught up with a new pose count still finds an image.
    for (let k = 0; k < MAX_POSES; k++) {
      const rect = placed[k % placed.length];
      if (rect) setRect(id, k, rect);
    }
  }
  return { rects, canvas };
}

export async function createRenderer(canvas) {
  if (!navigator.gpu) throw new Error(t('error.noWebgpu'));
  const adapter = await navigator.gpu.requestAdapter();
  if (!adapter) throw new Error(t('error.noAdapter'));
  const device = await adapter.requestDevice();
  const ctx = canvas.getContext('webgpu');
  const format = navigator.gpu.getPreferredCanvasFormat();
  ctx.configure({ device, format, alphaMode: 'opaque' });

  let [code, parts] = await Promise.all([
    fetch('src/shader.wgsl', { cache: 'no-cache' }).then((r) => r.text()),
    loadPartImages(BAKE),
  ]);
  const module = device.createShaderModule({ code });
  const info = await module.getCompilationInfo();
  const errors = info.messages.filter((m) => m.type === 'error');
  if (errors.length) {
    const detail = errors.map((m) => `${m.lineNum}:${m.linePos} ${m.message}`).join('; ');
    throw new Error(t('error.shader', { detail }));
  }

  const globals = new Float32Array(16);
  const globalsBuf = device.createBuffer({ size: globals.byteLength, usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST });
  const rectsBuf = device.createBuffer({ size: parts.rects.byteLength, usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST });
  device.queue.writeBuffer(rectsBuf, 0, parts.rects);
  const instBuf = device.createBuffer({ size: MAX_INSTANCES * INSTANCE_BYTES, usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST });

  let atlas = null;
  /** Upload the atlas canvas, making a new texture when its size changes. Returns true when it made one. */
  function uploadAtlas(src) {
    const size = src ? [src.width, src.height] : [1, 1];
    const fresh = !atlas || atlas.width !== size[0] || atlas.height !== size[1];
    if (fresh) {
      atlas?.destroy();
      atlas = device.createTexture({
        size,
        format: 'rgba8unorm',
        usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST | GPUTextureUsage.RENDER_ATTACHMENT,
      });
    }
    if (src) device.queue.copyExternalImageToTexture({ source: src }, { texture: atlas, premultipliedAlpha: false }, size);
    return fresh;
  }
  uploadAtlas(parts.canvas);
  const sampler = device.createSampler({ magFilter: 'linear', minFilter: 'linear' });

  const bindLayout = device.createBindGroupLayout({
    entries: [
      { binding: 0, visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT, buffer: { type: 'uniform' } },
      { binding: 1, visibility: GPUShaderStage.FRAGMENT, buffer: { type: 'uniform' } },
      { binding: 2, visibility: GPUShaderStage.FRAGMENT, texture: {} },
      { binding: 3, visibility: GPUShaderStage.FRAGMENT, sampler: {} },
    ],
  });
  const makeBindGroup = () => device.createBindGroup({
    layout: bindLayout,
    entries: [
      { binding: 0, resource: { buffer: globalsBuf } },
      { binding: 1, resource: { buffer: rectsBuf } },
      { binding: 2, resource: atlas.createView() },
      { binding: 3, resource: sampler },
    ],
  });
  let bindGroup = makeBindGroup();
  const layout = device.createPipelineLayout({ bindGroupLayouts: [bindLayout] });

  const bgPipeline = device.createRenderPipeline({
    layout,
    vertex: { module, entryPoint: 'vs_bg' },
    fragment: { module, entryPoint: 'fs_bg', targets: [{ format }] },
    primitive: { topology: 'triangle-list' },
  });
  const partPipeline = device.createRenderPipeline({
    layout,
    vertex: {
      module,
      entryPoint: 'vs',
      buffers: [{
        arrayStride: INSTANCE_BYTES,
        stepMode: 'instance',
        attributes: [
          { shaderLocation: 0, offset: 0, format: 'float32x4' },
          { shaderLocation: 1, offset: 16, format: 'float32x4' },
          { shaderLocation: 2, offset: 32, format: 'float32x4' },
        ],
      }],
    },
    fragment: {
      module,
      entryPoint: 'fs',
      targets: [{
        format,
        blend: {
          color: { srcFactor: 'src-alpha', dstFactor: 'one-minus-src-alpha' },
          alpha: { srcFactor: 'one', dstFactor: 'one-minus-src-alpha' },
        },
      }],
    },
    primitive: { topology: 'triangle-list' },
  });

  const view = { w: 1280, h: 720, px: 1 };

  /** Match the canvas to its CSS size. Returns the visible area in world units. */
  function resize() {
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const w = Math.max(1, Math.round(canvas.clientWidth * dpr));
    const h = Math.max(1, Math.round(canvas.clientHeight * dpr));
    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = w;
      canvas.height = h;
    }
    view.px = WORLD_HEIGHT / Math.min(w, h);
    view.w = w * view.px;
    view.h = h * view.px;
    return view;
  }

  /**
   * Draw one frame.
   * @param {ArrayBuffer} memory wasm linear memory
   * @param {number} ptr byte offset of the instance array
   * @param {number} count number of instances
   */
  function draw(memory, ptr, count, camX, camY, time) {
    globals.set([view.w, view.h, camX, camY, canvas.width, canvas.height, time, Math.floor(time * LOOK.stopMotionFps), view.px, LOOK.margin,
      0, 0, LOOK.shadowX, LOOK.shadowY, LOOK.shadowAlpha, LOOK.wobble]);
    device.queue.writeBuffer(globalsBuf, 0, globals);
    const n = Math.min(count, MAX_INSTANCES);
    if (n > 0) device.queue.writeBuffer(instBuf, 0, memory, ptr, n * INSTANCE_BYTES);

    const encoder = device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [{
        view: ctx.getCurrentTexture().createView(),
        loadOp: 'clear',
        storeOp: 'store',
        clearValue: { r: 0.92, g: 0.88, b: 0.79, a: 1 },
      }],
    });
    pass.setBindGroup(0, bindGroup);
    pass.setPipeline(bgPipeline);
    pass.draw(3);
    if (n > 0) {
      pass.setPipeline(partPipeline);
      pass.setVertexBuffer(0, instBuf);
      pass.draw(6, n);
    }
    pass.end();
    device.queue.submit([encoder.finish()]);
  }

  /** Re-bake the traced characters with new bake settings (see BAKE), for the debug panel. */
  async function rebake(bake) {
    const next = await loadPartImages(bake);
    if (uploadAtlas(next.canvas)) bindGroup = makeBindGroup();
    parts = next;
    device.queue.writeBuffer(rectsBuf, 0, parts.rects);
  }

  return { resize, draw, device, rebake };
}
