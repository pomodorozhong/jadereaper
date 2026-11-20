// Tests for the content generator: node --test tools/content.test.mjs (or npm test).

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { generate, validate } from './content.mjs';

const real = JSON.parse(await readFile(new URL('../content/game.json', import.meta.url), 'utf8'));
const copy = () => structuredClone(real);

test('the real content is valid and generates all three files', () => {
  assert.deepEqual(validate(real), []);
  const out = generate(real);
  assert.match(out.rust, /pub const SKILLS: \[SkillDef; \d+\]/);
  assert.match(out.js, /export const PART_IDS/);
  assert.match(out.wgsl, /fn isFlat/);
});

test('the committed files are up to date', async () => {
  const out = generate(real);
  for (const [key, path] of [['rust', 'core/src/content.rs'], ['js', 'src/content.js'], ['wgsl', 'src/content.wgsl']]) {
    assert.equal(await readFile(new URL(`../${path}`, import.meta.url), 'utf8'), out[key], `${path} is stale; run ./build.sh`);
  }
});

test('rejects a duplicate name', () => {
  const c = copy();
  c.skills.push({ ...c.skills[0] });
  assert.ok(validate(c).some((p) => p.includes('duplicate name "scythe"')));
  assert.throws(() => generate(c), /duplicate name/);
});

test('rejects a duplicate part id', () => {
  const c = copy();
  c.parts.push({ name: 'extra', id: c.parts[0].id });
  assert.ok(validate(c).some((p) => p.includes('duplicate id')));
});

test('rejects a duplicate event', () => {
  const c = copy();
  c.events.push(c.events[0]);
  assert.ok(validate(c).some((p) => p.includes('duplicate name')));
});

test('rejects references to undefined names', () => {
  const c = copy();
  c.kinds[0].moves = ['no_such_move'];
  c.kinds[1].rig = 'no_such_rig';
  c.rigs[0].parts[0].part = 'no_such_part';
  c.cards[0].target = 'no_such_skill';
  const problems = validate(c);
  assert.ok(problems.some((p) => p.includes('moves "no_such_move", which is not defined')));
  assert.ok(problems.some((p) => p.includes('rigs "no_such_rig", which is not defined')));
  assert.ok(problems.some((p) => p.includes('with a defined part name')));
  assert.ok(problems.some((p) => p.includes('"target" should be any, character, roll or a skill name')));
});

test('rejects unknown and missing fields, and bad values', () => {
  const c = copy();
  c.skills[0].speling = 1;
  delete c.skills[1].startup;
  c.skills[2].area = 'square';
  c.moves[0].after = { shot: { speed: 1 } };
  const problems = validate(c);
  assert.ok(problems.some((p) => p.includes('unknown field "speling"')));
  assert.ok(problems.some((p) => p.includes('missing "startup"')));
  assert.ok(problems.some((p) => p.includes('"area" should be one of')));
  assert.ok(problems.some((p) => p.includes('"after" should be')));
});
