import { test } from 'node:test';
import assert from 'node:assert/strict';
import { keyLabel, matches, type KeyPress } from '../src/commands.ts';

const press = (code: string, extra: Partial<KeyPress> = {}): KeyPress => ({
  code,
  ctrlKey: false,
  metaKey: false,
  shiftKey: false,
  altKey: false,
  ...extra,
});

test('shortcuts are matched by the physical key, so the Ukrainian layout works too', () => {
  // With the Ukrainian layout Ctrl+S reports the key "і", but its code is still KeyS.
  assert.equal(matches('Mod-S', press('KeyS', { ctrlKey: true }), false), true);
  assert.equal(matches('Mod-S', press('KeyS'), false), false);
  assert.equal(matches('Mod-S', press('KeyS', { ctrlKey: true, shiftKey: true }), false), false);
  assert.equal(matches('Mod-Shift-S', press('KeyS', { ctrlKey: true, shiftKey: true }), false), true);
  assert.equal(matches('Mod-Enter', press('Enter', { ctrlKey: true }), false), true);
  assert.equal(matches('Mod-/', press('Slash', { ctrlKey: true }), false), true);
  assert.equal(matches('Mod-B', press('KeyB', { ctrlKey: true, altKey: true }), false), false);
});

test('on a Mac, Mod is the command key', () => {
  assert.equal(matches('Mod-S', press('KeyS', { metaKey: true }), true), true);
  assert.equal(matches('Mod-S', press('KeyS', { ctrlKey: true }), true), false);
});

test('labels read Ctrl+Shift+S, or ⌘⇧S on a Mac', () => {
  assert.equal(keyLabel('Mod-Shift-S', false), 'Ctrl+Shift+S');
  assert.equal(keyLabel('Mod-Enter', false), 'Ctrl+Enter');
  assert.equal(keyLabel('Mod-/', false), 'Ctrl+/');
  assert.equal(keyLabel('Mod-Shift-S', true), '⌘⇧S');
  assert.equal(keyLabel('Mod-/', true), '⌘/');
});
