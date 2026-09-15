import { test } from 'node:test';
import assert from 'node:assert/strict';
import { text } from '../src/i18n.ts';

test('interface words fill in their blanks', () => {
  assert.equal(text('uk', 'position', { line: 4, column: 12 }), 'Рядок 4, стовпчик 12');
  assert.equal(text('en', 'position', { line: 4, column: 12 }), 'Ln 4, Col 12');
  assert.equal(
    text('uk', 'discard', { name: 'star.yu' }),
    'Незбережені зміни в «star.yu» пропадуть. Продовжити?',
  );
  assert.equal(text('en', 'menuFile'), 'File');
});
