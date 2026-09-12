import { test } from 'node:test';
import assert from 'node:assert/strict';
import { StringStream } from '@codemirror/language';
import { indentFor, yuParser } from '../src/yu-lang.ts';

const parser = yuParser(
  new Set(['якщо', 'повтори', 'разів', 'for']),
  new Set(['скажи', 'коло', 'circle']),
);

/** Every token of `line` that isn't whitespace, with its style. */
function tokens(line: string): [string, string | null][] {
  const stream = new StringStream(line, 4, 4);
  const out: [string, string | null][] = [];
  while (!stream.eol()) {
    const style = parser.token(stream, {});
    if (stream.current().trim()) out.push([stream.current(), style]);
    stream.start = stream.pos;
  }
  return out;
}

test('keywords, built-ins, text, numbers and comments are told apart', () => {
  assert.deepEqual(tokens('повтори 5 разів:  # п\'ять'), [
    ['повтори', 'keyword'],
    ['5', 'number'],
    ['разів', 'keyword'],
    [':', null],
    ['# п\'ять', 'comment'],
  ]);
  assert.deepEqual(tokens('коло(x, 2.5, "синій \\" 1")'), [
    ['коло', 'variableName.standard'],
    ['(', null],
    ['x', 'variableName'],
    [',', null],
    ['2.5', 'number'],
    [',', null],
    ['"синій \\" 1"', 'string'],
    [')', null],
  ]);
});

test('names may hold an apostrophe between letters', () => {
  assert.deepEqual(tokens("ім'я = 1"), [
    ["ім'я", 'variableName'],
    ['=', 'operator'],
    ['1', 'number'],
  ]);
});

test('a line ending in a colon opens a block', () => {
  assert.equal(indentFor('якщо x > 1:', 0, 4), 4);
  assert.equal(indentFor('    повтори 3 рази:  # коментар', 4, 4), 8);
  assert.equal(indentFor('    скажи(1)', 4, 4), 4);
});
