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
	  <label title="Loop every pose from start to end"><input type="checkbox" id="galPlay" checked> Loop</label>
	  <button type="button" id="galGame" title="Leave the gallery for the game, keeping the tuned values" style="all:unset;cursor:pointer;padding:0 6px;border:1px solid #b9ae95;border-radius:4px">Play the game</button></div>
	  ${row("progress", "Progress", 0, 1, 0.01)}
	  ${row("damage", "Hit damage", 0, 80, 1)}
	  ${row("angle", "Direction°", -180, 180, 5)}
	  <div style="color:#7a6f5c;max-width:19em">Direction: where skills aim and hits push (0 right, 90 down the screen). Tune each rig's numbers under "Pose" in the Debug panel.</div>
	  <div style="color:#7a6f5c;max-width:19em">Gamepad (or WASD and arrows): left and right scrub, up and down set the hit damage, the right stick points the direction, A starts or stops the loop.</div>`;
	box.addEventListener("keydown", (e) => e.stopPropagation());
	document.body.append(box);
	const progress = box.querySelector("#gal_progress");
	const show = (el) => {
		const v = view[el.dataset.key];
		el.nextElementSibling.textContent =
			el.dataset.key === "progress" ? v.toFixed(2) : v;
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
	// Leave the gallery for the game in this page: the tuned values stay, and the title screen shows again.
	let on = true;
	box.querySelector("#galGame").onclick = () => {
		on = false;
		core.jr_set_gallery(0, 0, 0, 0);
		style.remove();
		box.remove();
		for (const l of labels) l.el.remove();
		removeEventListener("resize", place);
		history.replaceState(null, "", "?debug");
	};

	const sync = () => {
		for (const el of box.querySelectorAll("input[type=range]")) {
			el.value = view[el.dataset.key];
			show(el);
		}
		box.querySelector("#galPlay").checked = view.play;
	};

	/** Drive the gallery from the gamepad, or WASD and the arrow keys (input.js `poll`): scrub, hit damage,
	 * direction, and the loop. */
	const usePad = (dt, pad) => {
		if (!pad) return;
		let moved = false;
		if (Math.abs(pad.mx) > 0.2) {
			view.play = false;
			view.progress = Math.min(
				1,
				Math.max(0, view.progress + pad.mx * dt * 0.6),
			);
			moved = true;
		}
		if (Math.abs(pad.my) > 0.2) {
			view.damage = Math.round(
				Math.min(80, Math.max(0, view.damage - pad.my * dt * 40)),
			);
			moved = true;
		}
		if (pad.aim?.kind === "stick" && Math.hypot(pad.aim.x, pad.aim.y) > 0.5) {
			view.angle =
				Math.round((Math.atan2(pad.aim.y, pad.aim.x) * 180) / Math.PI / 5) * 5;
			moved = true;
		}
		if (pad.confirm) {
			view.play = !view.play;
			moved = true;
		}
		if (moved) sync();
	};

	/** Call every frame before the core's update, with the polled input. Returns whether the gallery is showing. */
	return function tick(dt, pad) {
		if (!on) return false;
		usePad(dt, pad);
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
		return true;
	};
}
