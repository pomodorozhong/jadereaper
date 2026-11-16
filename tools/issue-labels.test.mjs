// Tests for tools/issue-labels.mjs against the real issue forms.
// Run with `npm test`.

import assert from 'node:assert/strict';
import { dirname, join } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import { FALLBACK, labelsFor, readForms } from './issue-labels.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const forms = readForms(join(ROOT, '.github', 'ISSUE_TEMPLATE'));

test('reads every form with its labels and required fields', () => {
  const byName = Object.fromEntries(forms.map((f) => [f.name, f]));
  assert.deepEqual(byName.Proposal.labels, ['enhancement', 'proposal']);
  assert.deepEqual(byName.Proposal.required, ['What to change']);
  assert.deepEqual(byName.Bug.labels, ['bug', 'needs-triage']);
  assert.deepEqual(byName.Bug.required, ['Environment', 'Steps to reproduce', 'Expected', 'Actual']);
  assert.deepEqual(byName.Question.labels, ['question']);
  assert.deepEqual(byName.Question.required, ['Question']);
});

test('a proposal body gets the proposal labels', () => {
  const body = '### What to change\n\nAdd a thing.\n\n### Why\n\nBecause.\n\n### Reference\n\n_No response_\n';
  assert.deepEqual(labelsFor(body, forms), ['enhancement', 'proposal']);
});

test('a bug body gets the bug labels, with Windows line endings too', () => {
  const body = ['### Environment', 'Chrome', '### Steps to reproduce', '1. Start', '### Expected', 'Works', '### Actual', 'Breaks'].join('\r\n');
  assert.deepEqual(labelsFor(body, forms), ['bug', 'needs-triage']);
});

test('a bug body missing a required section does not count as a bug', () => {
  const body = '### Steps to reproduce\n\n1. Start\n\n### Actual\n\nBreaks\n';
  assert.deepEqual(labelsFor(body, forms), FALLBACK);
});

test('a question body gets the question label', () => {
  const body = '### Question\n\nHow do I build it?\n\n### What you already tried or read\n\nThe README\n';
  assert.deepEqual(labelsFor(body, forms), ['question']);
});

test('a body that matches no form, or no body, gets the fallback', () => {
  assert.deepEqual(labelsFor('Just some text.\n## What to change\n', forms), FALLBACK);
  assert.deepEqual(labelsFor('', forms), FALLBACK);
  assert.deepEqual(labelsFor(null, forms), FALLBACK);
});
