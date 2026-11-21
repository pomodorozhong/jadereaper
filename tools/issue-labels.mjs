// Pick labels for an issue that was filed without an issue form, such as one
// made with `gh issue create` or the API. The Auto-label workflow
// (.github/workflows/auto-label.yml) calls this when a new issue has no labels.
//
// A form fills an issue with one `### <field label>` heading per field. An
// issue matches a form when its body has a heading for every required field of
// that form, and then gets the labels on the form's `labels:` line. The forms
// in .github/ISSUE_TEMPLATE/ stay the one place that says which labels a kind
// of issue gets. An issue that matches no form gets FALLBACK.

import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';

export const FALLBACK = ['needs-triage'];

// Read the parts of a form that matter here: its name, its labels and the
// labels of its required fields. Forms are simple enough that a line scan
// does; this avoids a YAML dependency in the workflow.
export function parseForm(text) {
  const form = { name: '', labels: [], required: [] };
  let field = null;
  const finish = () => {
    if (field?.required && field.label) form.required.push(field.label);
  };
  for (const line of text.split('\n')) {
    const name = line.match(/^name:\s*(.+)$/);
    const labels = line.match(/^labels:\s*(\[.*\])\s*$/);
    const label = field && line.match(/^\s+label:\s*(.+)$/);
    if (name) form.name = unquote(name[1]);
    else if (labels) form.labels = JSON.parse(labels[1]);
    else if (/^\s*- type:/.test(line)) {
      finish();
      field = { label: '', required: false };
    } else if (label) field.label = unquote(label[1]);
    else if (field && /^\s+required:\s*true\s*$/.test(line)) field.required = true;
  }
  finish();
  return form;
}

export function readForms(dir) {
  return readdirSync(dir)
    .filter((f) => f.endsWith('.yml') && f !== 'config.yml')
    .sort()
    .map((f) => parseForm(readFileSync(join(dir, f), 'utf8')))
    .filter((form) => form.labels.length && form.required.length);
}

export function labelsFor(body, forms) {
  const headings = new Set(
    (body ?? '')
      .split(/\r?\n/)
      .map((line) => line.match(/^###\s+(.+?)\s*$/)?.[1])
      .filter(Boolean),
  );
  const form = forms.find((f) => f.required.every((label) => headings.has(label)));
  return form ? form.labels : FALLBACK;
}

function unquote(s) {
  s = s.trim();
  return /^(["']).*\1$/.test(s) ? s.slice(1, -1) : s;
}
