// The ?debug panel's hit spin defaults (SPIN in src/main.js) must match the core's (Tune::DEFAULT and
// TUNE_KEYS in core/src/twist.rs): the panel sends each slider by its index in that order.
// node --test tools/spin.test.mjs (or npm test).

import { test } from "node:test";
import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const read = (path) => readFile(new URL(path, import.meta.url), "utf8");

/** `name: number` pairs between `start` and the next `end` in `text`, in order. */
function pairs(text, start, end) {
	const from = text.indexOf(start);
	assert.ok(from >= 0, `missing ${start}`);
	const body = text.slice(from + start.length, text.indexOf(end, from));
	return [...body.matchAll(/(\w+):\s*(-?[\d.]+)/g)].map((m) => [m[1], +m[2]]);
}

test("the panel's hit spin defaults match the core's, in the core's order", async () => {
	const [rust, js] = await Promise.all([
		read("../core/src/twist.rs"),
		read("../src/main.js"),
	]);
	const core = pairs(rust, "pub const DEFAULT: Tune = Tune {", "};");
	const keys = [
		...rust
			.slice(rust.indexOf("pub const TUNE_KEYS"))
			.split("];")[0]
			.matchAll(/"(\w+)"/g),
	].map((m) => m[1]);
	const panel = pairs(js, "const SPIN = {", "};");
	assert.equal(core.length, 9);
	assert.deepEqual(
		keys,
		core.map(([k]) => k),
	);
	assert.deepEqual(panel, core);
});
