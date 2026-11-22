// Host: loads the wasm core, runs the frame loop and ties input, sound,
// saving and the UI together. The game rules live in core/ (Rust).
import { createRenderer, LOOK } from "./renderer.js";
import { createInput } from "./input.js";
import { createAudio } from "./audio.js";
import { createUI } from "./ui.js";
import { loadSave, writeSave, recordRun } from "./save.js";
import {
	ANIM,
	ANIM_FLOATS,
	EVENT,
	STATE,
	STATE_FLOATS,
	SKILLS,
	CARDS,
	ELEMENTS,
} from "./data.js";
import { pickLang, setLang, getLang, nextLang } from "./i18n.js";
import { BAKE, SIZE, SIZE_ORDER } from "./characters.js";
import { RIGS, SHEET_KEYS } from "./content.js";
import { addGallery } from "./gallery.js";

/** Each monster's waddle: how far it tilts each way (degrees) and how often it changes pose (per second).
 * Tuned by the maintainer. Mirrors KINDS in core/src/defs.rs, which the core starts from; the ?debug panel sends
 * changes (jr_set_waddle). */
const WADDLE = {
	blotTilt: 2.25,
	blotRate: 2,
	wispTilt: 3.75,
	wispRate: 9,
	bruteTilt: 7.5,
	bruteRate: 3,
	eliteTilt: 7.5,
	eliteRate: 2.5,
};
/** The hit spin's numbers (core/src/twist.rs `Tune::DEFAULT`, in `TUNE_KEYS` order): the ?debug panel sends
 * changes with jr_set_spin. They change the run, since the spin decides how long a hit stops a character. */
const SPIN = {
	spin_share: 0.2,
	kick: 2,
	rate: 16,
	damping: 0.2,
	decel: 38,
	overshoot: 0.5,
	settle: 0.15,
	player_hold: 1,
	curve: 0.5,
};
const SPIN_KEYS = Object.keys(SPIN);
const WADDLE_KINDS = ["blot", "wisp", "brute", "elite"];
/** Reaction names by id (R_* in core/src/reactions.rs), for their text. */
const REACTIONS = [
	"",
	"steam",
	"overload",
	"collapse",
	"shatter",
	"stasis",
	"starfall",
];

/** ?debug: sliders for every visual setting. Bake settings re-draw the traced characters; look settings apply live. */
const DEBUG_SLIDERS = [
	["Bake (re-draws the characters)"],
	["bake", "lineWidth", "Line width", 0.3, 4, 0.1],
	["bake", "boilAmp", "Line boil", 0, 2, 0.05],
	["bake", "poses", "Boil poses", 1, 8, 1],
	["bake", "texels", "Sharpness", 1, 6, 0.5],
	["Look (live)"],
	["look", "boilFps", "Boil fps", 0, 24, 1],
	["look", "margin", "Paper margin", 0, 6, 0.1],
	["look", "shadowX", "Shadow x", -8, 8, 0.5],
	["look", "shadowY", "Shadow y", -8, 8, 0.5],
	["look", "shadowAlpha", "Shadow darkness", 0, 1, 0.05],
	["look", "thickness", "Paper thickness", 0, 8, 0.25],
	["look", "depthFade", "Depth fade", 0, 1, 0.05],
	["look", "depthDist", "Depth distance", 0.1, 2, 0.05],
	["look", "depthTone", "Depth tone (ground→shade)", 0, 1, 0.05],
	["look", "depthCurve", "Depth curve (power)", 0.2, 5, 0.05],
	["Size (drawn only; re-draws the characters)"],
	["size", "player", "Player", 0.5, 2, 0.05],
	["size", "blot", "Blot", 0.5, 2, 0.05],
	["size", "wisp", "Wisp", 0.5, 2, 0.05],
	["size", "brute", "Brute", 0.5, 2, 0.05],
	["size", "elite", "Elite", 0.5, 2, 0.05],
	["Waddle (live): tilt each way in degrees, and pose changes per second"],
	["waddle", "blotTilt", "Blot tilt", 0, 30, 0.25],
	["waddle", "blotRate", "Blot rate", 0, 15, 0.5],
	["waddle", "wispTilt", "Wisp tilt", 0, 30, 0.25],
	["waddle", "wispRate", "Wisp rate", 0, 15, 0.5],
	["waddle", "bruteTilt", "Brute tilt", 0, 30, 0.25],
	["waddle", "bruteRate", "Brute rate", 0, 15, 0.5],
	["waddle", "eliteTilt", "Elite tilt", 0, 30, 0.25],
	["waddle", "eliteRate", "Elite rate", 0, 15, 0.5],
	["Hit spin (live; changes how long a hit stops someone)"],
	["spin", "spin_share", "Spin at (share of max HP)", 0.05, 1, 0.01],
	["spin", "kick", "Swing just under a spin (rad)", 0, 3, 0.05],
	["spin", "rate", "Swing speed (rad/s)", 2, 40, 0.5],
	["spin", "damping", "Swing damping", 0.02, 1, 0.01],
	["spin", "decel", "Spin slowdown (rad/s²)", 5, 120, 1],
	["spin", "overshoot", "Spin overshoot (rad)", 0, 2, 0.05],
	["spin", "settle", "Moves again within (rad)", 0.02, 0.8, 0.01],
	["spin", "player_hold", "Player held past (rad)", 0, 3, 0.05],
	["spin", "curve", "Small-hit curve (1 = in proportion)", 0.1, 1, 0.05],
];
/** ?debug pose sliders: each rig's 3D pose numbers (`sheet` in content/game.json), sent with jr_set_pose. Each
 * slider is [min, max, step]; angles are in radians, lifts in world units. */
const POSE_SLIDERS = {
	aim: [0, 1.5, 0.05],
	charge: [-0.6, 0.8, 0.01],
	tremble: [0, 0.3, 0.01],
	strike: [-0.8, 0.4, 0.01],
	sag: [-0.8, 0.4, 0.01],
	sag_bend: [-1.2, 1.2, 0.01],
	pop: [0, 20, 0.5],
	grab: [0, 1.6, 0.01],
	shake: [0, 0.5, 0.01],
	roll: [0, 3, 1],
	bob: [0, 12, 0.5],
	sway: [0, 0.6, 0.01],
	flex: [0, 1, 0.05],
	walk_curl: [0, 0.6, 0.01],
};
/** The rigs with 3D poses: the player and the four small enemy kinds. */
const POSE_RIGS = RIGS.filter((r) => r.sheet);

/** Where the ?debug and ?gallery panels keep tuned values in this browser, so they carry across reloads and
 * between the gallery and the game. "Reset all" clears it; "Copy values" is how values become the defaults. */
const DEBUG_STORE = "jadereaper.debug";
function loadDebugValues() {
	try {
		const v = JSON.parse(localStorage.getItem(DEBUG_STORE) ?? "null");
		return v && typeof v === "object" ? v : {};
	} catch {
		return {};
	}
}
function saveDebugValues(values) {
	try {
		localStorage.setItem(DEBUG_STORE, JSON.stringify(values));
	} catch {}
}
function clearDebugValues() {
	try {
		localStorage.removeItem(DEBUG_STORE);
	} catch {}
}

function addPoseSliders(panel, core, saved, changed) {
	const pose = Object.fromEntries(
		POSE_RIGS.map((r) => [r.name, { ...r.sheet }]),
	);
	const btn =
		"all:unset;cursor:pointer;padding:0 6px;border:1px solid #b9ae95;border-radius:4px";
	const box = document.createElement("div");
	box.innerHTML = `<div style="margin-top:4px;color:#7a6f5c">Pose (live): <select id="dbgPoseRig">${POSE_RIGS.map((r) => `<option>${r.name}</option>`).join("")}</select></div>
    ${SHEET_KEYS.map((key) => {
			const [min, max, step] = POSE_SLIDERS[key];
			return `<div style="display:grid;grid-template-columns:9.5em 8em 2.6em auto;gap:6px;align-items:center">
    <label for="dbg_pose_${key}">${key.replace("_", " ")}</label><input id="dbg_pose_${key}" data-pose="${key}" type="range" min="${min}" max="${max}" step="${step}">
    <output style="font-variant-numeric:tabular-nums"></output><button type="button" data-pose-reset="${key}" style="${btn}">↺</button></div>`;
		}).join("")}`;
	panel.querySelector("#dbgBody").append(box);
	const pick = box.querySelector("#dbgPoseRig");
	const rig = () => POSE_RIGS[pick.selectedIndex];
	const sliders = [...box.querySelectorAll("[data-pose]")];
	const show = () => {
		for (const el of sliders) {
			const v = pose[rig().name][el.dataset.pose];
			el.value = v;
			el.nextElementSibling.textContent = v;
			el.parentElement.querySelector("[data-pose-reset]").title =
				`Reset to ${rig().sheet[el.dataset.pose]}`;
		}
	};
	const set = (r, key, v) => {
		pose[r.name][key] = v;
		core.jr_set_pose(r.id, SHEET_KEYS.indexOf(key), v);
	};
	for (const el of sliders) {
		el.oninput = () => {
			set(rig(), el.dataset.pose, +el.value);
			show();
			changed();
		};
		el.parentElement.querySelector("[data-pose-reset]").onclick = () => {
			set(rig(), el.dataset.pose, rig().sheet[el.dataset.pose]);
			show();
			changed();
		};
	}
	// Values saved in this browser from an earlier visit.
	for (const r of POSE_RIGS)
		for (const key of SHEET_KEYS) {
			const v = saved?.[r.name]?.[key];
			if (Number.isFinite(v) && v !== pose[r.name][key]) set(r, key, v);
		}
	pick.onchange = show;
	show();
	return {
		values: pose,
		resetAll() {
			for (const r of POSE_RIGS)
				for (const key of SHEET_KEYS) set(r, key, r.sheet[key]);
			show();
		},
	};
}

function addDebugPanel(renderer, core) {
	const sets = {
		bake: BAKE,
		look: LOOK,
		size: SIZE,
		waddle: WADDLE,
		spin: SPIN,
	};
	const defaults = {
		bake: { ...BAKE },
		look: { ...LOOK },
		size: { ...SIZE },
		waddle: { ...WADDLE },
		spin: { ...SPIN },
	};
	const panel = document.createElement("div");
	panel.style.cssText =
		"position:fixed;left:12px;bottom:12px;z-index:50;background:#fffdf6ee;border:1px solid #b9ae95;border-radius:6px;padding:6px 10px;font:12px system-ui,sans-serif;color:#2b2622;max-height:calc(100vh - 24px);overflow:auto";
	const btn =
		"all:unset;cursor:pointer;padding:0 6px;border:1px solid #b9ae95;border-radius:4px";
	const row = (
		set,
		key,
		label,
		min,
		max,
		step,
	) => `<div style="display:grid;grid-template-columns:9.5em 8em 2.6em auto;gap:6px;align-items:center">
    <label for="dbg_${set}_${key}">${label}</label><input id="dbg_${set}_${key}" data-set="${set}" data-key="${key}" type="range" min="${min}" max="${max}" step="${step}" value="${sets[set][key]}">
    <output style="font-variant-numeric:tabular-nums">${sets[set][key]}</output><button type="button" data-reset="${key}" title="Reset to ${defaults[set][key]}" style="${btn}">↺</button></div>`;
	panel.innerHTML = `<div style="display:flex;justify-content:space-between;gap:8px;margin-bottom:4px"><b>Debug</b><span>
    <label title="Spin every puppet in 3D: turn, lean, lift and bend, to check the paper sheets"><input type="checkbox" id="dbgSpin"> Spin puppets</label>
    <button type="button" id="dbgGallery" title="Open the pose gallery; tuned values carry over" style="${btn}">Gallery</button>
    <button type="button" id="dbgCopy" title="Copy the current values as JSON" style="${btn}">Copy values</button>
    <button type="button" id="dbgResetAll" title="Back to the defaults, and forget the values saved in this browser" style="${btn}">Reset all</button>
    <button type="button" id="dbgFold" style="${btn}">Hide</button></span></div>
    <div id="dbgBody" style="display:flex;flex-direction:column;gap:3px">${DEBUG_SLIDERS.map((r) => (r.length === 1 ? `<div style="margin-top:4px;color:#7a6f5c">${r[0]}</div>` : row(...r))).join("")}</div>`;
	// Keep game keys (WASD, Esc) from firing while a slider has focus.
	panel.addEventListener("keydown", (e) => e.stopPropagation());
	document.body.append(panel);
	const saved = loadDebugValues();
	// Tuned values are saved in this browser after every change, and loaded again on the next visit.
	let poses = null;
	const store = () =>
		saveDebugValues({
			bake: BAKE,
			look: LOOK,
			size: SIZE,
			waddle: WADDLE,
			spin: SPIN,
			pose: poses?.values,
		});
	poses = addPoseSliders(panel, core, saved.pose, store);

	// Re-bakes run one at a time; while one runs, only the latest wanted settings are kept.
	let baking = false,
		again = false;
	const rebake = async () => {
		if (baking) {
			again = true;
			return;
		}
		baking = true;
		do {
			again = false;
			await renderer.rebake({ ...BAKE });
		} while (again);
		baking = false;
	};
	const set = (el, v) => {
		const { set: which, key } = el.dataset;
		sets[which][key] = v;
		el.value = v;
		el.nextElementSibling.textContent = v;
		if (key === "boilFps" || key === "poses")
			core.jr_set_boil(LOOK.boilFps, BAKE.poses);
		if (which === "size") core.jr_set_draw_scale(SIZE_ORDER.indexOf(key), v);
		if (which === "waddle") {
			const kind = WADDLE_KINDS.findIndex((k) => key.startsWith(k));
			const name = WADDLE_KINDS[kind];
			core.jr_set_waddle(
				kind,
				(WADDLE[`${name}Tilt`] * Math.PI) / 180,
				WADDLE[`${name}Rate`],
			);
		}
		if (which === "spin") core.jr_set_spin(SPIN_KEYS.indexOf(key), v);
		if (which === "bake" || which === "size") rebake();
	};
	const sliders = [...panel.querySelectorAll("input[data-set]")];
	for (const el of sliders) {
		el.oninput = () => {
			set(el, +el.value);
			store();
		};
		el.parentElement.querySelector("[data-reset]").onclick = () => {
			set(el, defaults[el.dataset.set][el.dataset.key]);
			store();
		};
		// A value saved in this browser from an earlier visit.
		const v = saved[el.dataset.set]?.[el.dataset.key];
		if (Number.isFinite(v) && v !== sets[el.dataset.set][el.dataset.key])
			set(el, v);
	}
	panel.querySelector("#dbgResetAll").onclick = () => {
		for (const el of sliders) set(el, defaults[el.dataset.set][el.dataset.key]);
		poses.resetAll();
		clearDebugValues();
	};
	panel.querySelector("#dbgGallery").onclick = () => {
		location.search = "?gallery";
	};
	panel.querySelector("#dbgSpin").onchange = (e) =>
		core.jr_set_debug_spin(e.target.checked ? 1 : 0);
	panel.querySelector("#dbgCopy").onclick = () =>
		navigator.clipboard?.writeText(
			JSON.stringify(
				{
					bake: BAKE,
					look: LOOK,
					size: SIZE,
					waddle: WADDLE,
					spin: SPIN,
					pose: poses.values,
				},
				null,
				1,
			),
		);
	panel.querySelector("#dbgFold").onclick = (e) => {
		const body = panel.querySelector("#dbgBody");
		body.hidden = !body.hidden;
		e.target.textContent = body.hidden ? "Show" : "Hide";
	};
}

async function boot() {
	const canvas = document.getElementById("game");
	const save = loadSave();
	setLang(pickLang(save.lang));
	const audio = createAudio();
	audio.setVolume(save.volume);
	audio.setMuted(save.muted);

	// 'title' | 'pick' | 'playing' | 'levelup' | 'paused' | 'over'
	let mode = "title";
	// Skills picked for the run, in button order (left, right, Q).
	const picked = Array.isArray(save.loadout)
		? save.loadout.filter((id) => SKILLS[id]).slice(0, 3)
		: [];
	let settingsFrom = "title";

	const ui = createUI({ onAction });
	const input = createInput({ onKey, canvas });

	let renderer, core;
	try {
		renderer = await createRenderer(canvas);
		const bytes = await (
			await fetch("wasm/jadereaper_core.wasm", { cache: "no-cache" })
		).arrayBuffer();
		core = (await WebAssembly.instantiate(bytes, {})).instance.exports;
	} catch (err) {
		console.error(err);
		ui.error(err.message || String(err));
		return;
	}

	// The core picks each traced part's line boil pose, so it needs the rate and the number of baked poses.
	core.jr_set_boil(LOOK.boilFps, BAKE.poses);
	core.jr_set_reduce_motion(save.reduceMotion ? 1 : 0);
	SIZE_ORDER.forEach((who, i) => {
		core.jr_set_draw_scale(i, SIZE[who]);
	});
	// ?gallery shows every rig in every pose instead of the game, with the ?debug panel to tune them.
	const params = new URLSearchParams(location.search);
	const gallery = params.has("gallery") ? addGallery(core, canvas) : null;
	if (params.has("debug") || gallery) addDebugPanel(renderer, core);

	const seed = () => (Math.random() * 0xffffffff) >>> 0;
	// Views are rebuilt on every read because wasm memory can grow and detach them.
	const readState = () =>
		new Float32Array(core.memory.buffer, core.jr_state_ptr(), STATE_FLOATS);
	/** Each character's animation state (core/src/anim.rs), as objects keyed by the ANIM field names. The core
	 * poses the puppets from the same records (rigs.rs); this copy is for scripted checks. */
	const readAnims = () => {
		const n = core.jr_anim_count();
		const a = new Float32Array(
			core.memory.buffer,
			core.jr_anims_ptr(),
			n * ANIM_FLOATS,
		);
		return Array.from({ length: n }, (_, i) =>
			Object.fromEntries(
				Object.entries(ANIM).map(([k, at]) => [k, a[i * ANIM_FLOATS + at]]),
			),
		);
	};
	const three = (s, at) => Array.from(s.subarray(at, at + 3));
	// A card offer waiting for a confirm: the slot of an element card that would replace another element.
	let armed = -1;
	const offer = (s) => ({
		choices: three(s, STATE.CHOICE0),
		slots: three(s, STATE.CHOICE_SLOT0),
		kind: s[STATE.PICK],
		loadout: three(s, STATE.LOADOUT0),
		elements: three(s, STATE.ELEMENT0),
		armed,
	});
	/** True when taking offered card i would replace another element on its skill, so it needs a second pick. */
	function needsConfirm(s, i) {
		const card = CARDS[s[STATE.CHOICE0 + i]];
		const slot = s[STATE.CHOICE_SLOT0 + i];
		if (!card || card.element === "none" || slot < 0) return false;
		const old = s[STATE.ELEMENT0 + slot];
		return old > 0 && ELEMENTS[old - 1].name !== card.element;
	}

	function toTitle() {
		core.jr_reset(seed());
		mode = "title";
		ui.showHud(false);
		ui.records(save);
		ui.show("title");
	}

	function pickSkills() {
		mode = "pick";
		ui.showHud(false);
		ui.skillPick(picked);
	}

	/** Back to the skill pick after a run (Restart, Try again), with the last picks still chosen. The run behind
	 * the menu is reset, as on the title, so the old one does not play on under it. */
	function pickAgain() {
		core.jr_reset(seed());
		pickSkills();
	}

	function startRun() {
		if (picked.length !== 3) {
			pickSkills();
			return;
		}
		core.jr_start(seed());
		core.jr_set_loadout(...picked);
		core.jr_offer_passive();
		core.jr_update(0);
		armed = -1;
		mode = "playing";
		ui.showHud(true);
		ui.show(null);
	}

	function setPaused(paused) {
		if (paused && mode === "playing") {
			mode = "paused";
			ui.show("pause");
		} else if (!paused && mode === "paused") {
			mode = "playing";
			ui.show(null);
		}
	}

	function saveSettings() {
		save.volume = audio.volume;
		save.muted = audio.muted;
		writeSave(save);
		ui.settings(save);
	}

	function onAction(action, value) {
		audio.unlock();
		audio.play(action === "choose" || action === "start" ? "confirm" : "ui");
		switch (action) {
			case "start":
				pickSkills();
				break;
			case "pick-skill": {
				const id = Number(value);
				const at = picked.indexOf(id);
				if (at >= 0) picked.splice(at, 1);
				else if (picked.length < 3) picked.push(id);
				ui.skillPick(picked);
				break;
			}
			case "pick-begin":
				save.loadout = [...picked];
				writeSave(save);
				startRun();
				break;
			case "again":
				pickAgain();
				break;
			case "resume":
				setPaused(false);
				break;
			case "restart":
				pickAgain();
				break;
			case "quit":
				toTitle();
				break;
			case "settings":
				settingsFrom = ui.screen;
				ui.show("settings");
				break;
			case "back":
				ui.show(settingsFrom);
				break;
			case "lang":
				setLang(nextLang(getLang()));
				save.lang = getLang();
				saveSettings();
				ui.records(save);
				break;
			case "mute":
				audio.setMuted(!audio.muted);
				saveSettings();
				break;
			case "motion":
				save.reduceMotion = !save.reduceMotion;
				core.jr_set_reduce_motion(save.reduceMotion ? 1 : 0);
				saveSettings();
				break;
			case "vol-up":
				audio.setVolume(audio.volume + 0.1);
				saveSettings();
				audio.play("confirm");
				break;
			case "vol-down":
				audio.setVolume(audio.volume - 0.1);
				saveSettings();
				audio.play("confirm");
				break;
			case "clear-save":
				Object.assign(save, {
					bestRoom: 0,
					wins: 0,
					fastestWin: 0,
					runs: 0,
					totalKills: 0,
					legacy: null,
				});
				writeSave(save);
				ui.records(save);
				break;
			case "choose":
				if (mode === "levelup") {
					const i = Number(value);
					if (needsConfirm(readState(), i) && armed !== i) {
						armed = i;
						ui.cards(offer(readState()));
						break;
					}
					armed = -1;
					core.jr_choose(i);
					afterChoice();
				}
				break;
		}
	}

	/** The core may offer another choice straight away if several levels were gained at once. */
	function afterChoice() {
		core.jr_update(0);
		const s = readState();
		if (s[STATE.CHOOSING]) {
			ui.cards(offer(s));
		} else {
			mode = "playing";
			ui.show(null);
		}
	}

	function back() {
		if (ui.screen === "settings") onAction("back");
		else if (mode === "paused") setPaused(false);
	}

	function onKey(key) {
		audio.unlock();
		if (key.pause) {
			if (mode === "playing") setPaused(true);
			else back();
		} else if (key.nav && ui.screen) {
			if (ui.navigate(key.nav[0], key.nav[1])) audio.play("ui");
		} else if (key.pick !== undefined && mode === "levelup" && !ui.locked) {
			const card = document.querySelectorAll("#cards .card")[key.pick];
			if (card) card.click();
		} else if (key.mute) {
			audio.setMuted(!audio.muted);
			saveSettings();
		}
	}

	window.addEventListener("pointerdown", () => audio.unlock());
	window.addEventListener("blur", () => setPaused(true));
	document.addEventListener("visibilitychange", () => {
		if (document.hidden) setPaused(true);
	});

	ui.settings(save);
	toTitle();

	/** The aim as an offset from the player in world units: the mouse's ground point, or the stick's direction. */
	function aimOffset(aim, s, view) {
		if (!aim) return [0, 0];
		if (aim.kind === "stick") return [aim.x * 200, aim.y * 200];
		const r = canvas.getBoundingClientRect();
		const wx = s[STATE.CAM_X] + ((aim.x - r.left) / r.width - 0.5) * view.w;
		const wy = s[STATE.CAM_Y] + ((aim.y - r.top) / r.height - 0.5) * view.h;
		return [wx - s[STATE.PLAYER_X], wy - s[STATE.PLAYER_Y]];
	}

	let last = performance.now();

	function frame(now) {
		const dt = Math.min(0.05, Math.max(0, (now - last) / 1000));
		last = now;

		const view = renderer.resize(readState()[STATE.ZOOM]);
		core.jr_set_view(view.w, view.h);

		const pad = input.poll(dt);
		ui.setDevice(pad.device);
		// While the pose gallery shows, the pad drives it, not the hidden menus behind it.
		const inGallery = gallery?.(dt, pad) ?? false;
		if (pad.pause && !inGallery) onKey({ pause: true });
		if (pad.back && !inGallery) back();
		if (ui.screen && !inGallery) {
			if (pad.nav && ui.navigate(pad.nav[0], pad.nav[1])) audio.play("ui");
			if (pad.confirm && !ui.locked) {
				audio.unlock();
				ui.confirm();
			}
		}

		// Paused freezes everything. Other menus keep the paper puppets moving.
		const step = mode === "paused" ? 0 : dt;
		if (mode === "playing") {
			const [ax, ay] = aimOffset(pad.aim, readState(), view);
			core.jr_input(pad.mx, pad.my, ax, ay, pad.buttons);
		} else {
			core.jr_input(0, 0, 0, 0, 0);
		}
		core.jr_update(step);

		const s = readState();
		if (mode === "playing") {
			const count = core.jr_event_count();
			if (count) {
				const ev = new Float32Array(
					core.memory.buffer,
					core.jr_events_ptr(),
					count * 4,
				);
				const r = canvas.getBoundingClientRect();
				for (let i = 0; i < count; i++) {
					audio.play(ev[i * 4]);
					if (ev[i * 4] === EVENT.HIT || ev[i * 4] === EVENT.REACTION) {
						// Lift the number from the ground point to about chest height.
						const sx =
							r.left +
							((ev[i * 4 + 1] - s[STATE.CAM_X]) / view.w + 0.5) * r.width;
						const sy =
							r.top +
							((ev[i * 4 + 2] - 36 - s[STATE.CAM_Y]) / view.h + 0.5) * r.height;
						if (ev[i * 4] === EVENT.HIT) ui.damage(sx, sy, ev[i * 4 + 3]);
						else ui.reaction(sx, sy, REACTIONS[ev[i * 4 + 3]] ?? "");
					}
				}
			}
			ui.hud({
				hp: s[STATE.HP],
				maxHp: s[STATE.MAX_HP],
				room: s[STATE.ROOM],
				rooms: s[STATE.ROOMS],
				wave: s[STATE.WAVE],
				waves: s[STATE.WAVES],
				boss: s[STATE.BOSS],
				time: s[STATE.TIME],
				kills: s[STATE.KILLS],
				loadout: three(s, STATE.LOADOUT0),
				elements: three(s, STATE.ELEMENT0),
				owned: Array.from(
					s.subarray(STATE.OWNED0, STATE.OWNED0 + CARDS.length),
				),
				bossBar: s[STATE.BOSS_KIND]
					? {
							kind: s[STATE.BOSS_KIND] - 1,
							health: s[STATE.BOSS_HP],
							max: s[STATE.BOSS_MAX],
							meter: s[STATE.BOSS_METER],
							flags: s[STATE.BOSS_FLAGS],
							stance: s[STATE.STANCE],
							finisher: s[STATE.FINISHER],
						}
					: null,
				grabbed: s[STATE.ACT] === 5,
			});
			if (s[STATE.OVER]) {
				mode = "over";
				const won = !!s[STATE.WON];
				// Rooms cleared: the room reached, plus the last one when the run is won.
				const run = {
					rooms: s[STATE.ROOM] + (won ? 1 : 0),
					kills: s[STATE.KILLS],
					time: s[STATE.TIME],
					won,
				};
				ui.gameOver(run, save, recordRun(save, run));
			} else if (s[STATE.CHOOSING]) {
				mode = "levelup";
				armed = -1;
				ui.cards(offer(s));
			}
		}

		renderer.draw(
			core.memory.buffer,
			core.jr_instances_ptr(),
			core.jr_instance_count(),
			s[STATE.CAM_X],
			s[STATE.CAM_Y],
		);
		requestAnimationFrame(frame);
	}
	requestAnimationFrame(frame);

	// Handy in the console: jadereaper.core, jadereaper.state(), jadereaper.anims().
	window.jadereaper = {
		core,
		state: () => Array.from(readState()),
		anims: readAnims,
		get mode() {
			return mode;
		},
	};
}

boot();
