// Paper UI: screens, HUD and menu focus. All DOM work lives here.
import { SKILLS, SLOT_KEYS, CARDS, ELEMENTS, KINDS } from "./data.js";
import { formatTime } from "./save.js";
import { t, getLang, setText } from "./i18n.js";

const $ = (id) => document.getElementById(id);
const SCREENS = [
	"title",
	"settings",
	"skillpick",
	"levelup",
	"pause",
	"over",
	"error",
];
const INPUT_LOCK_MS = 300;

export function createUI({ onAction }) {
	let current = null;
	let lockedUntil = 0;
	const cache = {};

	document.addEventListener("click", (e) => {
		const btn = e.target.closest("button[data-action]");
		if (!btn || performance.now() < lockedUntil) return;
		onAction(btn.dataset.action, btn.dataset.value);
	});

	function buttons() {
		return current ? Array.from($(current).querySelectorAll("button")) : [];
	}

	// The screen button that last had focus, so the pad can take focus back after something outside the screen
	// (a ?debug slider) took it.
	let lastFocused = null;
	// While the gamepad moves focus, show the highlight (style.css `.pad-focus`); the mouse turns it off again.
	const padFocus = () => document.body.classList.add("pad-focus");
	window.addEventListener("pointermove", () =>
		document.body.classList.remove("pad-focus"),
	);
	document.addEventListener("focusin", (e) => {
		if (current && $(current).contains(e.target)) lastFocused = e.target;
	});
	/** The focused button of the open screen, taking focus back from outside the screen if needed. */
	function focusedButton() {
		const list = buttons();
		const el = document.activeElement;
		if (list.includes(el)) return el;
		const back = list.includes(lastFocused) ? lastFocused : list[0];
		back?.focus({ preventScroll: true });
		return back ?? null;
	}

	/** Show one screen (or none). A short lock stops a held button from picking something by accident. */
	function show(name) {
		for (const id of SCREENS) $(id).hidden = id !== name;
		current = name || null;
		if (current) {
			lockedUntil = performance.now() + INPUT_LOCK_MS;
			const first = buttons()[0];
			if (first) first.focus({ preventScroll: true });
		} else if (document.activeElement instanceof HTMLElement) {
			document.activeElement.blur();
		}
	}

	/** The nearest button from `from` in direction (dx, dy) on screen, so the pad moves through a grid by rows
	 * and columns. Null when there is none that way. */
	function nearestToward(from, list, dx, dy) {
		const centre = (el) => {
			const r = el.getBoundingClientRect();
			return [r.left + r.width / 2, r.top + r.height / 2];
		};
		const [fx, fy] = centre(from);
		const box = from.getBoundingClientRect();
		// A button counts as that way only when its centre is past the focused button's edge: a focused card is
		// lifted and tilted (style.css), so its neighbours in the row sit a little below its centre.
		const reach = Math.abs(dx) * (box.width / 2) + Math.abs(dy) * (box.height / 2);
		let best = null;
		let bestScore = Infinity;
		for (const el of list) {
			if (el === from) continue;
			const [x, y] = centre(el);
			// Distance along the pressed direction, and across it; buttons off to the side count for less.
			const along = (x - fx) * dx + (y - fy) * dy;
			const across = Math.abs((x - fx) * dy - (y - fy) * dx);
			if (along <= reach) continue;
			const score = along + across * 2;
			if (score < bestScore) {
				bestScore = score;
				best = el;
			}
		}
		return best;
	}

	/** Move focus through the open screen's buttons: to the nearest one in the pressed direction, or the next or
	 * previous in order (wrapping round) when there is none that way. Disabled buttons are skipped. Returns true
	 * if focus moved. */
	function navigate(dx, dy) {
		const list = buttons().filter((el) => !el.disabled);
		if (!list.length) return false;
		padFocus();
		const from = focusedButton();
		const i = list.indexOf(from);
		let next = i < 0 ? list[0] : nearestToward(from, list, dx, dy);
		if (!next) {
			const step = dx > 0 || dy > 0 ? 1 : -1;
			next = list[(i + step + list.length) % list.length];
		}
		next.focus({ preventScroll: true });
		return true;
	}

	/** Click the focused button (gamepad confirm). */
	function confirm() {
		if (current) padFocus();
		const el = current ? focusedButton() : null;
		if (el instanceof HTMLButtonElement) {
			el.click();
			return true;
		}
		return false;
	}

	function set(id, prop, value) {
		const key = id + prop;
		if (cache[key] === value) return;
		cache[key] = value;
		if (prop === "text") $(id).textContent = value;
		else $(id).style.setProperty(prop, value);
	}

	const rgb = (c) => `rgb(${c.map((v) => Math.round(v * 255)).join(" ")})`;

	/** A skill button chip: the skill's glyph, its element's glyph in the element's colour, and the button. */
	function skillChip(id, element, slot) {
		const chip = document.createElement("div");
		chip.className = "chip";
		const sk = SKILLS[id];
		chip.title = sk ? t(`skill.${sk.name}.name`) : "";
		const glyph = document.createElement("b");
		glyph.textContent = sk?.glyph ?? "?";
		chip.append(glyph);
		if (element > 0) {
			const el = ELEMENTS[element - 1];
			const mark = document.createElement("i");
			mark.className = "element";
			mark.textContent = el.glyph;
			mark.style.color = rgb(el.color);
			mark.title = t(`element.${el.name}`);
			chip.append(mark);
		}
		const key = document.createElement("small");
		key.textContent = SLOT_KEYS[slot];
		chip.append(key);
		return chip;
	}

	function hud({
		hp,
		maxHp,
		room,
		rooms,
		wave,
		waves,
		boss,
		time,
		kills,
		loadout,
		elements,
		owned,
		bossBar,
		grabbed,
	}) {
		$("bossBar").hidden = !bossBar;
		if (bossBar) {
			const { kind, health, max, meter, flags } = bossBar;
			set("bossName", "text", t(`boss.${KINDS[kind].name}`));
			$("bossName").classList.toggle("enraged", !!(flags & 1));
			$("bossBar").classList.toggle("down", !!(flags & 2));
			set(
				"bossFill",
				"width",
				`${Math.max(0, Math.min(100, (health / max) * 100)).toFixed(1)}%`,
			);
			set(
				"bossMeter",
				"width",
				`${(flags & 2 ? 100 : meter * 100).toFixed(1)}%`,
			);
			// The Dragon's stance shows its weakness, the other element.
			const { stance, finisher } = bossBar;
			$("bossWeak").hidden = !stance;
			if (stance) {
				const weak = ELEMENTS.find(
					(e) => e.name === (stance === 1 ? "ice" : "fire"),
				);
				set(
					"bossWeak",
					"text",
					t("hud.weak", { element: t(`element.${weak.name}`) }),
				);
				$("bossWeak").style.color = rgb(weak.color);
			}
			$("finisherPrompt").hidden = !(finisher > 0);
			if (finisher > 0) set("finisherPrompt", "text", t("hud.finisher"));
		} else {
			$("finisherPrompt").hidden = true;
		}
		$("grabPrompt").hidden = !grabbed;
		const skillSig = getLang() + loadout.join(",") + elements.join(",");
		if (cache.skills !== skillSig) {
			cache.skills = skillSig;
			$("skills").replaceChildren(
				...loadout.map((id, slot) => skillChip(id, elements[slot], slot)),
			);
		}
		set(
			"hpFill",
			"width",
			`${Math.max(0, Math.min(100, (hp / maxHp) * 100)).toFixed(1)}%`,
		);
		set("hpText", "text", String(Math.ceil(hp)));
		set(
			"roomFill",
			"width",
			`${Math.max(0, Math.min(100, ((room + Math.min(wave, waves) / waves) / rooms) * 100)).toFixed(1)}%`,
		);
		const at = {
			room: room + 1,
			rooms,
			wave: Math.min(wave + 1, waves),
			waves,
		};
		set("roomText", "text", t(boss ? "hud.boss" : "hud.room", at));
		set("timeText", "text", formatTime(time));
		set("killsText", "text", String(kills));
		// Character and roll cards held, with how many of each.
		const sig = getLang() + owned.join(",");
		if (cache.chips !== sig) {
			cache.chips = sig;
			$("chips").replaceChildren(
				...owned.flatMap((n, id) => {
					const card = CARDS[id];
					if (!n || !card || !["character", "roll"].includes(card.target))
						return [];
					const chip = document.createElement("div");
					chip.className = "chip";
					chip.title = t(`card.${card.name}.name`);
					const glyph = document.createElement("b");
					glyph.textContent = card.glyph;
					chip.append(glyph, n > 1 ? String(n) : "");
					return [chip];
				}),
			);
		}
	}

	function showHud(visible) {
		$("hud").hidden = !visible;
		if (!visible) $("numbers").replaceChildren();
	}

	/** A reaction's name floating up from a screen point, in the colour of the reaction. */
	function reaction(x, y, name) {
		const layer = $("numbers");
		const el = document.createElement("span");
		el.className = "reaction";
		el.textContent = t(`reaction.${name}`);
		el.style.left = `${x}px`;
		el.style.top = `${y - 24}px`;
		el.addEventListener("animationend", () => el.remove());
		layer.append(el);
	}

	/** A damage number floating up from a screen point. */
	function damage(x, y, value) {
		const layer = $("numbers");
		if (layer.childElementCount > 40) layer.firstElementChild.remove();
		const el = document.createElement("span");
		el.textContent = String(Math.round(value));
		if (value >= 25) el.className = "big";
		el.style.left = `${x}px`;
		el.style.top = `${y}px`;
		el.addEventListener("animationend", () => el.remove());
		layer.append(el);
	}

	/**
	 * Offer cards: the starting passive (kind 1) or a reward (kind 2). `slots` gives the skill button each card goes
	 * on (-1 for none), `elements` each button's element, `armed` the slot of a card waiting for a confirm (an
	 * element card that would replace another element), or -1.
	 */
	function cards({ choices, slots, kind, loadout, elements, armed }) {
		setText($("cardsBanner"), t(kind === 1 ? "cards.passive" : "cards.reward"));
		$("cards").replaceChildren(
			...choices.flatMap((id, i) => {
				const c = CARDS[id];
				if (id < 0 || !c) return [];
				const slot = slots[i];
				const card = document.createElement("button");
				card.className = "card";
				card.dataset.action = "choose";
				card.dataset.value = String(i);
				const target =
					slot >= 0
						? t("cards.on", {
								skill: t(`skill.${SKILLS[loadout[slot]].name}.name`),
							})
						: t(c.target === "roll" ? "cards.roll" : "cards.character");
				const el =
					c.element !== "none"
						? ELEMENTS.find((e) => e.name === c.element)
						: null;
				const old = slot >= 0 && el ? elements[slot] : 0;
				const replaces =
					old > 0 && ELEMENTS[old - 1].name !== c.element
						? t(`element.${ELEMENTS[old - 1].name}`)
						: "";
				const parts = [
					["key", String(i + 1)],
					["glyph", c.glyph],
					["name", t(`card.${c.name}.name`)],
					["target", target],
					["desc", t(`card.${c.name}.desc`)],
					[
						"rank",
						replaces
							? t(armed === i ? "cards.confirm" : "cards.replaces", {
									element: replaces,
								})
							: "",
					],
				];
				for (const [cls, text] of parts) {
					if (!text) continue;
					const span = document.createElement("span");
					span.className = cls;
					span.textContent = text;
					if (cls === "glyph" && el) span.style.color = rgb(el.color);
					card.append(span);
				}
				if (armed === i) card.classList.add("armed");
				return [card];
			}),
		);
		if (current !== "levelup") show("levelup");
		else
			$("cards")
				.querySelector(`[data-value="${armed}"]`)
				?.focus({ preventScroll: true });
	}

	/** The skill pick: every skill as a card; the picked ones show which button they go on, in pick order. */
	function skillPick(picked) {
		const focused = document.activeElement?.dataset?.value;
		$("skillCards").replaceChildren(
			...SKILLS.map((sk) => {
				const card = document.createElement("button");
				card.className = "card";
				card.dataset.action = "pick-skill";
				card.dataset.value = String(sk.id);
				const slot = picked.indexOf(sk.id);
				if (slot >= 0) card.classList.add("picked");
				const parts = [
					["key", slot >= 0 ? SLOT_KEYS[slot] : ""],
					["glyph", sk.glyph],
					["name", t(`skill.${sk.name}.name`)],
					["weight", t(`weight.${sk.weight}`)],
					["desc", t(`skill.${sk.name}.desc`)],
				];
				for (const [cls, text] of parts) {
					const span = document.createElement("span");
					span.className = cls;
					span.textContent = text;
					card.append(span);
				}
				return card;
			}),
		);
		$("pickBegin").disabled = picked.length !== 3;
		setText($("pickBegin"), t("pick.begin", { n: picked.length }));
		if (current !== "skillpick") show("skillpick");
		else if (focused)
			$("skillCards")
				.querySelector(`[data-value="${focused}"]`)
				?.focus({ preventScroll: true });
	}

	function gameOver({ rooms, kills, time, won }, save, beaten) {
		setText($("overTitle"), t(won ? "over.won" : "over.title"));
		const rows = [
			[t("over.rooms"), String(rooms), beaten.rooms],
			[t("over.kills"), String(kills), false],
			[t("over.time"), formatTime(time), beaten.win],
		];
		if (save.fastestWin)
			rows.push([t("over.fastest"), formatTime(save.fastestWin), false]);
		$("stats").replaceChildren(
			...rows.flatMap(([label, value, best]) => {
				const dt = document.createElement("dt");
				dt.textContent = label;
				const dd = document.createElement("dd");
				dd.textContent = value;
				if (best) dd.className = "best";
				return [dt, dd];
			}),
		);
		$("recordStamp").hidden = !(beaten.rooms || beaten.win);
		show("over");
	}

	function records(save) {
		$("records").textContent = save.runs
			? t(save.runs === 1 ? "records.one" : "records.other", {
					rooms: save.bestRoom,
					wins: save.wins,
					runs: save.runs,
				})
			: "";
	}

	function settings({ volume, muted, reduceMotion }) {
		setText(
			$("motionBtn"),
			t(reduceMotion ? "settings.motionOff" : "settings.motionOn"),
		);
		const on = Math.round(volume * 10);
		$("volPips").replaceChildren(
			...Array.from({ length: 10 }, (_, i) => {
				const pip = document.createElement("i");
				if (i < on) pip.className = "on";
				return pip;
			}),
		);
		for (const btn of document.querySelectorAll('button[data-action="mute"]')) {
			btn.textContent = t(muted ? "settings.soundOff" : "settings.soundOn");
		}
		setText($("langBtn"), t("settings.language", { name: t("lang.name") }));
	}

	function error(message) {
		$("errorText").textContent = message;
		show("error");
	}

	return {
		show,
		navigate,
		confirm,
		hud,
		showHud,
		damage,
		reaction,
		skillPick,
		cards,
		gameOver,
		records,
		settings,
		error,
		get screen() {
			return current;
		},
		get locked() {
			return performance.now() < lockedUntil;
		},
	};
}
