// WebGPU renderer: one background pass and one instanced draw of paper parts.
import { PART_IDS } from './data.js';

const INSTANCE_BYTES = 48;
const MAX_INSTANCES = 4096;
const WORLD_HEIGHT = 640; // world units across the shorter screen side
const STOP_MOTION_FPS = 8;

/** Load optional part images listed in assets/parts.json and pack them into one atlas canvas. */
async function loadPartImages() {
  const rects = new Float32Array(32 * 4);
  let manifest = {};
  try {
    const res = await fetch('assets/parts.json', { cache: 'no-cache' });
    if (res.ok) manifest = await res.json();
  } catch {
    // No manifest: every part is drawn in code.
  }
  const entries = Object.entries(manifest).filter(([name]) => name in PART_IDS);
  if (!entries.length) return { rects, canvas: null };

  const images = await Promise.all(entries.map(async ([name, path]) => {
    try {
      const blob = await (await fetch(`assets/${path}`)).blob();
      return { id: PART_IDS[name], bmp: await createImageBitmap(blob) };
    } catch (err) {
      console.warn(`Could not load part image for "${name}"`, err);
      return null;
    }
  }));

  const SIZE = 2048;
  const GAP = 4;
  const canvas = document.createElement('canvas');
  canvas.width = canvas.height = SIZE;
  const ctx = canvas.getContext('2d');
  let x = GAP, y = GAP, rowH = 0;
  for (const img of images) {
    if (!img) continue;
    // Shrink anything larger than a quarter of the atlas.
    const scale = Math.min(1, 512 / Math.max(img.bmp.width, img.bmp.height));
    const w = Math.round(img.bmp.width * scale), h = Math.round(img.bmp.height * scale);
    if (x + w + GAP > SIZE) { x = GAP; y += rowH + GAP; rowH = 0; }
    if (y + h + GAP > SIZE) { console.warn('Part atlas is full, skipping an image'); continue; }
    ctx.drawImage(img.bmp, x, y, w, h);
    rects.set([x / SIZE, y / SIZE, w / SIZE, h / SIZE], img.id * 4);
    x += w + GAP;
    rowH = Math.max(rowH, h);
  }
  return { rects, canvas };
}

export async function createRenderer(canvas) {
  if (!navigator.gpu) throw new Error('WebGPU is not available in this browser.');
  const adapter = await navigator.gpu.requestAdapter();
  if (!adapter) throw new Error('No WebGPU adapter was found.');
  const device = await adapter.requestDevice();
  const ctx = canvas.getContext('webgpu');
  const format = navigator.gpu.getPreferredCanvasFormat();
  ctx.configure({ device, format, alphaMode: 'opaque' });

  const [code, parts] = await Promise.all([
    fetch('src/shader.wgsl', { cache: 'no-cache' }).then((r) => r.text()),
    loadPartImages(),
  ]);
  const module = device.createShaderModule({ code });
  const info = await module.getCompilationInfo();
  const errors = info.messages.filter((m) => m.type === 'error');
  if (errors.length) {
    throw new Error('Shader failed to compile: ' + errors.map((m) => `${m.lineNum}:${m.linePos} ${m.message}`).join('; '));
  }

  const globals = new Float32Array(12);
  const globalsBuf = device.createBuffer({ size: globals.byteLength, usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST });
  const rectsBuf = device.createBuffer({ size: parts.rects.byteLength, usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST });
  device.queue.writeBuffer(rectsBuf, 0, parts.rects);
  const instBuf = device.createBuffer({ size: MAX_INSTANCES * INSTANCE_BYTES, usage: GPUBufferUsage.VERTEX | GPUBufferUsage.COPY_DST });

  const atlasSize = parts.canvas ? [parts.canvas.width, parts.canvas.height] : [1, 1];
  const atlas = device.createTexture({
    size: atlasSize,
    format: 'rgba8unorm',
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST | GPUTextureUsage.RENDER_ATTACHMENT,
  });
  if (parts.canvas) {
    device.queue.copyExternalImageToTexture({ source: parts.canvas }, { texture: atlas, premultipliedAlpha: false }, atlasSize);
  }
  const sampler = device.createSampler({ magFilter: 'linear', minFilter: 'linear' });

  const bindLayout = device.createBindGroupLayout({
    entries: [
      { binding: 0, visibility: GPUShaderStage.VERTEX | GPUShaderStage.FRAGMENT, buffer: { type: 'uniform' } },
      { binding: 1, visibility: GPUShaderStage.FRAGMENT, buffer: { type: 'uniform' } },
      { binding: 2, visibility: GPUShaderStage.FRAGMENT, texture: {} },
      { binding: 3, visibility: GPUShaderStage.FRAGMENT, sampler: {} },
    ],
  });
  const bindGroup = device.createBindGroup({
    layout: bindLayout,
    entries: [
      { binding: 0, resource: { buffer: globalsBuf } },
      { binding: 1, resource: { buffer: rectsBuf } },
      { binding: 2, resource: atlas.createView() },
      { binding: 3, resource: sampler },
    ],
  });
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
    globals.set([view.w, view.h, camX, camY, canvas.width, canvas.height, time, Math.floor(time * STOP_MOTION_FPS), view.px]);
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

  return { resize, draw, device };
}
