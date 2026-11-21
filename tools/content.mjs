// Content generator: reads content/game.json, the one file that defines the game's parts, events, enemy kinds,
// skills and moves, checks it, and writes the tables and ids for each side of the game:
//
//   core/src/content.rs   Rust consts and tables for the simulation
//   src/content.js        the same ids and tables for the host and UI
//   src/content.wgsl      part ids and the flat-part test, put in front of src/shader.wgsl by the renderer
//
//     node tools/content.mjs           write the three files (./build.sh runs this first)
//     node tools/content.mjs --check   fail if they are out of date with content/game.json
//
// It rejects duplicate names or ids, references to names that are not defined, unknown fields and missing ones.
// Each table's fields and their types are in TABLES below; add a field there before using it in game.json.

import { readFile, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SOURCE = "content/game.json";
const OUT = {
	rust: "core/src/content.rs",
	js: "src/content.js",
	wgsl: "src/content.wgsl",
};
/** Part ids must stay below core's PART_SLOTS. */
const PART_SLOTS = 32;

/** Enum values in game.json, mapped to their Rust names. `weight` is stored as a number. */
const ENUMS = {
	weight: { light: 0, medium: 1, heavy: 2 },
	SkillKind: ["melee", "shot", "nova", "orbit", "leap"],
	AreaKind: ["circle", "arc", "line", "ring"],
	Special: [
		"none",
		"behind",
		"sweep",
		"grab",
		"summon",
		"eat",
		"pools",
		"lances",
		"field",
		"shards",
		"dive",
		"finisher",
	],
	Stance: ["any", "fire", "ice"],
	Effect: [
		"none",
		"recovery",
		"reach",
		"damage",
		"stagger",
		"max_hp",
		"speed",
		"regen",
		"might",
		"armor",
		"roll_dist",
		"trail",
	],
};
/** A card's target: one of these, or the name of a skill. */
const TARGETS = {
	any: "Target::AnySkill",
	character: "Target::Character",
	roll: "Target::Roll",
};

/**
 * The tables in game.json besides parts and events. `fields` maps each field to its type, or to [type, default]
 * for a field that may be left out:
 * f32, u8, bool, str (JavaScript only), f32x2, f32x3, part (a part name), ref:<table> or refs:<table> (names in
 * another table), waves:<table> (a list of lists of names), opt:<table> ("none" or a name in another table; 0 or id + 1 in Rust), enum:<name>, target (a
 * card's target), after (a move's after-effect), pose (a rig pose) and rigparts (a rig's parts).
 */
const TABLES = {
	rigs: {
		rust: "Rig",
		prefix: "RG_",
		id: "usize",
		fields: {
			parts: "rigparts",
			windup: "pose",
			attack: "pose",
			hurt: "pose",
			roll: "pose",
		},
	},
	kinds: {
		rust: "Kind",
		prefix: "K_",
		id: "u8",
		fields: {
			rig: "ref:rigs",
			hp: "f32",
			speed: "f32",
			r: "f32",
			rate: "f32",
			tilt: "f32",
			poise: "f32",
			boss: "bool",
			moves: "refs:moves",
		},
	},
	skills: {
		rust: "SkillDef",
		prefix: "SK_",
		id: "usize",
		fields: {
			glyph: "str",
			weight: "enum:weight",
			kind: "enum:SkillKind",
			startup: "f32",
			active: "f32",
			recovery: "f32",
			cancel: "f32",
			damage: "f32",
			stagger: "f32",
			knockback: "f32",
			area: "enum:AreaKind",
			reach: "f32",
			width: "f32",
			hyper: "bool",
			dash: "f32",
			combo: "u8",
			walk: "f32",
			lasts: "f32",
		},
	},
	elements: {
		rust: "Element",
		prefix: "E_",
		id: "u8",
		fields: { glyph: "str", color: "f32x3" },
	},
	cards: {
		rust: "Card",
		prefix: "C_",
		id: "usize",
		fields: {
			glyph: "str",
			target: "target",
			element: "opt:elements",
			effect: "enum:Effect",
			amount: "f32",
			passive: "bool",
		},
	},
	rooms: {
		rust: "Room",
		prefix: "R_",
		id: "usize",
		fields: { w: "f32", h: "f32", boss: "opt:kinds", waves: "waves:kinds" },
	},
	moves: {
		rust: "Move",
		prefix: "M_",
		id: "usize",
		fields: {
			windup: "f32",
			active: "f32",
			recover: "f32",
			area: "enum:AreaKind",
			reach: "f32",
			width: "f32",
			damage: "f32",
			dash: "f32",
			after: "after",
			range: "f32x2",
			chain: "u8",
			// Boss moves only.
			weight: ["f32", 1],
			cooldown: ["f32", 0],
			phases: ["phases", [1, 2]],
			big: ["bool", false],
			special: ["enum:Special", "none"],
			knockdown: ["bool", false],
			stance: ["enum:Stance", "any"],
		},
	},
};

const upper = (name) => name.toUpperCase();
const pascal = (name) =>
	name.replace(/(^|_)(\w)/g, (_, __, c) => c.toUpperCase());
const f32 = (x) => (Number.isInteger(x) ? `${x}.0` : String(x));
const isNum = (x) => typeof x === "number" && Number.isFinite(x);
const NAME = /^[a-z][a-z0-9_]*$/;

/** Check the content. Returns a list of problems, empty when it is fine. */
export function validate(content) {
	const problems = [];
	const names = {};
	const unique = (table, list, key = "name") => {
		const seen = new Set();
		for (const [i, item] of list.entries()) {
			const name = key ? item?.[key] : item;
			if (typeof name !== "string" || !NAME.test(name))
				problems.push(
					`${table}[${i}]: name must be lower_snake_case, got ${JSON.stringify(name)}`,
				);
			else if (seen.has(name))
				problems.push(`${table}: duplicate name "${name}"`);
			seen.add(name);
		}
		names[table] = seen;
	};
	for (const key of ["parts", "events", ...Object.keys(TABLES)]) {
		if (!Array.isArray(content?.[key]))
			problems.push(`missing the "${key}" list`);
	}
	const extra = Object.keys(content ?? {}).filter(
		(k) => !["parts", "events", ...Object.keys(TABLES)].includes(k),
	);
	for (const k of extra) problems.push(`unknown top-level list "${k}"`);
	if (problems.length) return problems;

	unique("parts", content.parts);
	const ids = new Set();
	for (const p of content.parts) {
		if (!Number.isInteger(p.id) || p.id < 0 || p.id >= PART_SLOTS)
			problems.push(
				`parts: "${p.name}" needs an id from 0 to ${PART_SLOTS - 1}`,
			);
		else if (ids.has(p.id))
			problems.push(`parts: duplicate id ${p.id} ("${p.name}")`);
		ids.add(p.id);
		for (const k of Object.keys(p))
			if (!["name", "id", "flat"].includes(k))
				problems.push(`parts: "${p.name}" has unknown field "${k}"`);
	}
	unique("events", content.events, null);
	for (const table of Object.keys(TABLES)) unique(table, content[table]);

	for (const [table, def] of Object.entries(TABLES)) {
		for (const item of content[table]) {
			const where = `${table}: "${item.name}"`;
			for (const k of Object.keys(item))
				if (k !== "name" && !(k in def.fields))
					problems.push(`${where} has unknown field "${k}"`);
			for (const [field, spec] of Object.entries(def.fields)) {
				const [type, fallback] = Array.isArray(spec) ? spec : [spec];
				const v = item[field] ?? fallback;
				if (v === undefined) {
					problems.push(`${where} is missing "${field}"`);
					continue;
				}
				const bad = (want) =>
					problems.push(
						`${where}: "${field}" should be ${want}, got ${JSON.stringify(v)}`,
					);
				if (type === "f32") {
					if (!isNum(v)) bad("a number");
				} else if (type === "u8") {
					if (!Number.isInteger(v) || v < 0 || v > 255)
						bad("a whole number from 0 to 255");
				} else if (type === "bool") {
					if (typeof v !== "boolean") bad("true or false");
				} else if (type === "str") {
					if (typeof v !== "string") bad("a string");
				} else if (type === "f32x2" || type === "f32x3") {
					const n = Number(type.slice(4));
					if (!Array.isArray(v) || v.length !== n || !v.every(isNum))
						bad(`${n} numbers`);
				} else if (type === "part") {
					if (!names.parts.has(v))
						problems.push(
							`${where}: "${field}" names part "${v}", which is not defined`,
						);
				} else if (type.startsWith("ref:")) {
					const target = type.slice(4);
					if (!names[target].has(v))
						problems.push(
							`${where}: "${field}" names ${target} "${v}", which is not defined`,
						);
				} else if (type.startsWith("refs:")) {
					const target = type.slice(5);
					if (!Array.isArray(v) || !v.length) bad(`a list of ${target} names`);
					else
						for (const r of v)
							if (!names[target].has(r))
								problems.push(
									`${where}: "${field}" names ${target} "${r}", which is not defined`,
								);
				} else if (type.startsWith("enum:")) {
					const values = ENUMS[type.slice(5)];
					const ok = Array.isArray(values) ? values.includes(v) : v in values;
					if (!ok)
						bad(
							`one of ${(Array.isArray(values) ? values : Object.keys(values)).join(", ")}`,
						);
				} else if (type.startsWith("waves:")) {
					const target = type.slice(6);
					if (
						!Array.isArray(v) ||
						!v.length ||
						!v.every((w) => Array.isArray(w) && w.length)
					)
						bad(`a list of waves, each a list of ${target} names`);
					else
						for (const w of v)
							for (const r of w)
								if (!names[target].has(r))
									problems.push(
										`${where}: "${field}" names ${target} "${r}", which is not defined`,
									);
				} else if (type === "phases") {
					if (
						!Array.isArray(v) ||
						!v.length ||
						!v.every((n) => n === 1 || n === 2)
					)
						bad("a list of boss phases, 1 or 2");
				} else if (type === "pose") {
					const keys = ["dx", "dy", "tilt", "sx", "sy"];
					if (
						!v ||
						typeof v !== "object" ||
						Object.keys(v).length !== keys.length ||
						!keys.every((k) => isNum(v[k]))
					)
						bad(`{ ${keys.join(", ")} } numbers`);
				} else if (type === "rigparts") {
					const keys = {
						part: "part",
						x: "num",
						y: "num",
						hw: "num",
						hh: "num",
						rot: "num",
						color: "rgb",
						held: "bool",
					};
					if (!Array.isArray(v) || !v.length) bad("a list of parts");
					else
						for (const [i, rp] of v.entries()) {
							const ok =
								rp &&
								typeof rp === "object" &&
								Object.keys(rp).length === Object.keys(keys).length &&
								Object.entries(keys).every(([k, t]) =>
									t === "num"
										? isNum(rp[k])
										: t === "bool"
											? typeof rp[k] === "boolean"
											: t === "rgb"
												? Array.isArray(rp[k]) &&
													rp[k].length === 3 &&
													rp[k].every(isNum)
												: names.parts.has(rp[k]),
								);
							if (!ok)
								problems.push(
									`${where}: part ${i} needs ${Object.keys(keys).join(", ")}, with a defined part name`,
								);
						}
				} else if (type === "target") {
					if (!(v in TARGETS) && !names.skills.has(v))
						problems.push(
							`${where}: "${field}" should be any, character, roll or a skill name, got ${JSON.stringify(v)}`,
						);
				} else if (type.startsWith("opt:")) {
					const target = type.slice(4);
					if (v !== "none" && !names[target].has(v))
						problems.push(
							`${where}: "${field}" names ${target} "${v}", which is not defined`,
						);
				} else if (type === "after") {
					const ok =
						v === "none" ||
						(v?.shot &&
							isNum(v.shot.speed) &&
							isNum(v.shot.r) &&
							Object.keys(v).length === 1) ||
						(v?.wave &&
							isNum(v.wave.to) &&
							isNum(v.wave.speed) &&
							isNum(v.wave.band) &&
							Object.keys(v).length === 1);
					if (!ok)
						bad(
							'"none", { "shot": { speed, r } } or { "wave": { to, speed, band } }',
						);
				}
			}
		}
	}
	return problems;
}

function rustValue(type, v) {
	if (type === "f32") return f32(v);
	if (type === "u8") return String(v);
	if (type === "bool") return String(v);
	if (type === "f32x2") return `(${v.map(f32).join(", ")})`;
	if (type === "f32x3") return `[${v.map(f32).join(", ")}]`;
	if (type === "part") return `P_${upper(v)}`;
	if (type.startsWith("ref:"))
		return `${TABLES[type.slice(4)].prefix}${upper(v)}`;
	if (type.startsWith("refs:"))
		return `&[${v.map((r) => `${TABLES[type.slice(5)].prefix}${upper(r)}`).join(", ")}]`;
	if (type === "enum:weight") return String(ENUMS.weight[v]);
	if (type.startsWith("enum:")) return `${type.slice(5)}::${pascal(v)}`;
	if (type.startsWith("waves:")) {
		const prefix = TABLES[type.slice(6)].prefix;
		return `&[${v.map((w) => `&[${w.map((r) => `${prefix}${upper(r)}`).join(", ")}]`).join(", ")}]`;
	}
	if (type === "phases")
		return String(v.reduce((m, n) => m | (1 << (n - 1)), 0));
	if (type === "pose")
		return `Pose { dx: ${f32(v.dx)}, dy: ${f32(v.dy)}, tilt: ${f32(v.tilt)}, sx: ${f32(v.sx)}, sy: ${f32(v.sy)} }`;
	if (type === "rigparts") {
		return `&[${v.map((p) => `RigPart { part: P_${upper(p.part)}, x: ${f32(p.x)}, y: ${f32(p.y)}, hw: ${f32(p.hw)}, hh: ${f32(p.hh)}, rot: ${f32(p.rot)}, color: [${p.color.map(f32).join(", ")}], held: ${p.held} }`).join(", ")}]`;
	}
	if (type === "target") return TARGETS[v] ?? `Target::Skill(SK_${upper(v)})`;
	if (type.startsWith("opt:"))
		return v === "none"
			? "0"
			: `${TABLES[type.slice(4)].prefix}${upper(v)} + 1`;
	if (type === "after") {
		if (v === "none") return "After::None";
		if (v.shot)
			return `After::Shot { speed: ${f32(v.shot.speed)}, r: ${f32(v.shot.r)} }`;
		return `After::Wave { to: ${f32(v.wave.to)}, speed: ${f32(v.wave.speed)}, band: ${f32(v.wave.band)} }`;
	}
	throw new Error(`no Rust form for ${type}`);
}

const HEADER = (comment) =>
	`${comment} Generated by tools/content.mjs from ${SOURCE}. Do not edit; change ${SOURCE} and run ./build.sh.\n`;

/** Generate the three files. Throws when the content has problems. */
export function generate(content) {
	const problems = validate(content);
	if (problems.length)
		throw new Error(`${SOURCE} has problems:\n  ${problems.join("\n  ")}`);

	// Rust.
	const rs = [
		HEADER("//!"),
		"#![allow(clippy::all)]",
		"",
		"use crate::defs::Kind;",
		"use crate::moves::{After, Move, Special, Stance};",
		"use crate::skills::{AreaKind, SkillDef, SkillKind};",
		"use crate::cards::{Card, Effect, Element, Target};",
		"use crate::rooms::Room;",
		"use crate::rigs::{Pose, Rig, RigPart};",
		"",
	];
	rs.push("// Part ids.");
	for (const p of content.parts)
		rs.push(`pub const P_${upper(p.name)}: f32 = ${f32(p.id)};`);
	rs.push(
		`/// Every defined part id.`,
		`pub const PARTS: [f32; ${content.parts.length}] = [${content.parts.map((p) => `P_${upper(p.name)}`).join(", ")}];`,
	);
	rs.push("", "// Event types, read by the host for sound and damage numbers.");
	content.events.forEach((e, i) => {
		rs.push(`pub const EV_${upper(e)}: f32 = ${f32(i + 1)};`);
	});
	for (const [table, def] of Object.entries(TABLES)) {
		const list = content[table];
		rs.push("", `// ${table}`);
		list.forEach((item, i) => {
			rs.push(`pub const ${def.prefix}${upper(item.name)}: ${def.id} = ${i};`);
		});
		rs.push(`pub const N_${upper(table)}: usize = ${list.length};`);
		rs.push(`pub const ${upper(table)}: [${def.rust}; ${list.length}] = [`);
		for (const item of list) {
			const fields = Object.entries(def.fields)
				.map(([f, spec]) =>
					Array.isArray(spec)
						? [f, spec[0], item[f] ?? spec[1]]
						: [f, spec, item[f]],
				)
				.filter(([, t]) => t !== "str")
				.map(([f, t, v]) => `${f}: ${rustValue(t, v)}`);
			rs.push(
				`    // ${item.name}`,
				`    ${def.rust} { ${fields.join(", ")} },`,
			);
		}
		rs.push("];");
	}

	// JavaScript.
	const js = [
		HEADER("//"),
		"/** Part names and ids. Names can be given an image in assets/parts.json. */",
	];
	js.push(
		`export const PART_IDS = ${JSON.stringify(Object.fromEntries(content.parts.map((p) => [p.name, p.id])), null, 2)};`,
		"",
	);
	js.push(
		`export const EVENT = ${JSON.stringify(Object.fromEntries(content.events.map((e, i) => [upper(e), i + 1])), null, 2)};`,
	);
	for (const table of Object.keys(TABLES)) {
		js.push(
			"",
			`/** ${table}, indexed by id. */`,
			`export const ${upper(table)} = ${JSON.stringify(
				content[table].map((item, id) => ({ id, ...item })),
				null,
				2,
			)};`,
		);
	}

	// WGSL.
	const wgsl = [HEADER("//"), "// Part ids."];
	for (const p of content.parts)
		wgsl.push(`const P_${upper(p.name)}: i32 = ${p.id};`);
	const flat = content.parts
		.filter((p) => p.flat)
		.map((p) => `part == P_${upper(p.name)}`);
	wgsl.push(
		"",
		"// Parts drawn as flat ink or wash: no outline, margin or shadow.",
		`fn isFlat(part: i32) -> bool {`,
		`  return ${flat.join(" || ") || "false"};`,
		"}",
	);

	return {
		rust: `${rs.join("\n")}\n`,
		js: `${js.join("\n")}\n`,
		wgsl: `${wgsl.join("\n")}\n`,
	};
}

async function main() {
	const check = process.argv.includes("--check");
	const content = JSON.parse(await readFile(join(ROOT, SOURCE), "utf8"));
	let out;
	try {
		out = generate(content);
	} catch (err) {
		console.error(err.message);
		process.exit(1);
	}
	const stale = [];
	for (const [key, path] of Object.entries(OUT)) {
		const file = join(ROOT, path);
		const now = await readFile(file, "utf8").catch(() => "");
		if (now === out[key]) continue;
		if (check) stale.push(path);
		else await writeFile(file, out[key]);
	}
	if (stale.length) {
		console.error(
			`Out of date with ${SOURCE}: ${stale.join(", ")}. Run ./build.sh and commit them.`,
		);
		process.exit(1);
	}
	console.log(
		check
			? "content: up to date"
			: `content: wrote ${Object.values(OUT).join(", ")}`,
	);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) await main();
