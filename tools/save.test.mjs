// Tests for the save migration in src/save.js: node --test tools/save.test.mjs (or npm test).

import { test } from "node:test";
import assert from "node:assert/strict";
import { migrate, recordRun, VERSION } from "../src/save.js";

test("a version 1 save migrates, keeping settings, runs and kills", () => {
	const v1 = {
		bestTime: 312,
		bestKills: 900,
		bestLevel: 17,
		runs: 12,
		totalKills: 4000,
		volume: 0.3,
		muted: true,
		lang: "zh-Hant",
	};
	const save = migrate(v1);
	assert.equal(save.version, VERSION);
	assert.equal(save.runs, 12);
	assert.equal(save.totalKills, 4000);
	assert.equal(save.volume, 0.3);
	assert.equal(save.muted, true);
	assert.equal(save.lang, "zh-Hant");
	assert.equal(save.bestRoom, 0);
	assert.equal(save.wins, 0);
	assert.deepEqual(save.legacy, {
		bestTime: 312,
		bestKills: 900,
		bestLevel: 17,
	});
	assert.equal(save.bestTime, undefined);
});

test("a current save stays as it is", () => {
	const now = migrate({
		version: VERSION,
		bestRoom: 4,
		wins: 1,
		fastestWin: 600,
		runs: 3,
		totalKills: 90,
		loadout: [0, 5, 6],
	});
	assert.equal(now.bestRoom, 4);
	assert.deepEqual(now.loadout, [0, 5, 6]);
	assert.equal(now.legacy, null);
});

test("broken data gives the defaults", () => {
	for (const bad of [null, "x", [1, 2], 7])
		assert.equal(migrate(bad).version, VERSION);
});

test("a run is folded into the records", () => {
	const save = migrate({ version: VERSION });
	globalThis.localStorage ??= { setItem() {} };
	assert.deepEqual(
		recordRun(save, { rooms: 3, kills: 40, time: 200, won: false }),
		{ rooms: true, win: false },
	);
	assert.deepEqual(
		recordRun(save, { rooms: 6, kills: 80, time: 700, won: true }),
		{ rooms: true, win: true },
	);
	assert.deepEqual(
		recordRun(save, { rooms: 6, kills: 80, time: 800, won: true }),
		{ rooms: false, win: false },
	);
	assert.equal(save.wins, 2);
	assert.equal(save.fastestWin, 700);
	assert.equal(save.runs, 3);
	assert.equal(save.totalKills, 200);
});
