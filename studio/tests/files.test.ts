import { test } from 'node:test';
import assert from 'node:assert/strict';
import { baseName, pictureName } from '../src/files.ts';

test('a picture is named after its program', () => {
  assert.equal(pictureName('star.yu', 'svg'), 'star.svg');
  assert.equal(pictureName('мій малюнок.yu', 'png'), 'мій малюнок.png');
  assert.equal(pictureName('без назви.yu', 'png'), 'без назви.png');
});

test('a base name drops only the last extension', () => {
  assert.equal(baseName('star.yu'), 'star');
  assert.equal(baseName('star.old.yu'), 'star.old');
  assert.equal(baseName('program'), 'program');
  assert.equal(baseName('.yu'), '.yu');
});
