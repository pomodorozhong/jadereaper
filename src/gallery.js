// The pose gallery (?gallery): every rig in every pose in one grid, drawn by the core (core/src/gallery.rs), for
// tuning the 3D pose numbers with the ?debug pose sliders. This adds the grid's labels and the gallery's own
// controls: play, how far through each pose, the hit's damage and the direction skills aim and hits push.
import { RIGS } from "./content.js";

/** The gallery's columns, in the order the core draws them (GALLERY_POSES in core/src/gallery.rs). */
const GALLERY_POSES = [
	"idle",
	"walk",
	"charge",
	"strike",
	"recovery",
	"hit",
	"down",
	"grabbed",
	"roll",
	"stunned",
	"turn round",
];
/** Shares of the view kept for the labels and a bottom margin; GALLERY_TOP, GALLERY_LEFT and GALLERY_BOTTOM in
 * core/src/gallery.rs. */
const TOP = 0.08;
const LEFT = 0.08;
const BOTTOM = 0.04;
/** Seconds for one pass through every pose while playing. */
const PLAY_TIME = 1.6;

export function addGallery(core, canvas) {
	const view = { play: true, progress: 0, damage: 20, angle: 30 };
	const style = document.createElement("style");
	style.textContent = `.screen, #hud, #numbers { display: none !important; }
	.gallery-label { position: fixed; z-index: 40; transform: translate(-50%, -50%); font: 12px system-ui, sans-serif;
	  color: #2b2622; background: #fffdf6cc; padding: 0 4px; border-radius: 3px; pointer-events: none; white-space: nowrap; }`;
	document.head.append(style);

	const labels = [
		...GALLERY_POSES.map((name, c) => ({
			name,
			x: LEFT + ((c + 0.5) * (1 - LEFT)) / GALLERY_POSES.length,
			y: TOP * 0.5,
		})),
		...RIGS.map((r, i) => ({
			name: r.name,
			x: LEFT * 0.5,
			y: TOP + ((i + 0.6) * (1 - TOP - BOTTOM)) / RIGS.length,
		})),
	].map((l) => {
		const el = document.createElement("div");
		el.className = "gallery-label";
		el.textContent = l.name;
		document.body.append(el);
		return { ...l, el };
	});
	const place = () => {
		const r = canvas.getBoundingClientRect();
		for (const l of labels) {
			l.el.style.left = `${r.left + l.x * r.width}px`;
			l.el.style.top = `${r.top + l.y * r.height}px`;
		}
	};
	addEventListener("resize", place);
	place();

	const box = document.createElement("div");
	box.style.cssText =
		"position:fixed;right:12px;bottom:12px;z-index:50;background:#fffdf6ee;border:1px solid #b9ae95;border-radius:6px;padding:6px 10px;font:12px system-ui,sans-serif;color:#2b2622;display:flex;flex-direction:column;gap:3px";
	const row = (key, label, min, max, step) =>
		`<div style="display:grid;grid-template-columns:6.5em 9em 3em;gap:6px;align-items:center"><label for="gal_${key}">${label}</label><input id="gal_${key}" data-key="${key}" type="range" min="${min}" max="${max}" step="${step}" value="${view[key]}"><output>${view[key]}</output></div>`;
	box.innerHTML = `<div style="display:flex;justify-content:space-between;gap:8px"><b>Pose gallery</b>
	  <label title="Loop every pose from start to end"><input type="checkbox" id="galPlay" checked> Play</label></div>
	  ${row("progress", "Progress", 0, 1, 0.01)}
	  ${row("damage", "Hit damage", 0, 80, 1)}
	  ${row("angle", "Direction°", -180, 180, 5)}
	  <div style="color:#7a6f5c;max-width:19em">Direction: where skills aim and hits push (0 right, 90 down the screen). Tune each rig's numbers under "Pose" in the Debug panel.</div>`;
	box.addEventListener("keydown", (e) => e.stopPropagation());
	document.body.append(box);
	const progress = box.querySelector("#gal_progress");
	const show = (el) => {
		el.nextElementSibling.textContent = view[el.dataset.key];
	};
	for (const el of box.querySelectorAll("input[type=range]")) {
		el.oninput = () => {
			view[el.dataset.key] = +el.value;
			show(el);
			if (el === progress) {
				// Scrubbing stops the playback.
				view.play = false;
				box.querySelector("#galPlay").checked = false;
			}
		};
	}
	box.querySelector("#galPlay").onchange = (e) => {
		view.play = e.target.checked;
	};

	/** Call every frame before the core's update. */
	return function tick(dt) {
		if (view.play) {
			view.progress = (view.progress + dt / PLAY_TIME) % 1;
			progress.value = view.progress;
			progress.nextElementSibling.textContent = view.progress.toFixed(2);
		}
		core.jr_set_gallery(
			1,
			view.progress,
			view.damage,
			(view.angle * Math.PI) / 180,
		);
	};
}
