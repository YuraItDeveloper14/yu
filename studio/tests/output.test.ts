import { test } from 'node:test';
import assert from 'node:assert/strict';
import { tabAfterRun } from '../src/output.ts';

test('a drawing opens Picture; errors and plain text open Terminal', () => {
  assert.equal(tabAfterRun({ ok: true, drew: true }), 'picture');
  assert.equal(tabAfterRun({ ok: true, drew: false }), 'terminal');
  assert.equal(tabAfterRun({ ok: false, drew: true }), 'terminal');
  assert.equal(tabAfterRun({ ok: false, drew: false }), 'terminal');
});
