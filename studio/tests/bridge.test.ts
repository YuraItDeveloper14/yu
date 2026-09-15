import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { loadYu, type Hooks } from '../src/bridge.ts';

const wasm = readFileSync(new URL('../public/yu_wasm.wasm', import.meta.url));
const quiet: Hooks = { print: () => {}, ask: () => null };

test('a program prints and draws', async () => {
  const out: string[] = [];
  const yu = await loadYu(wasm, { print: (line) => out.push(line), ask: () => null });
  const r = yu.run('скажи("Привіт")\nколо(300, 200, 50)', 'uk', [1, 0]);
  assert.equal(r.ok, true);
  assert.equal(r.drew, true);
  assert.match(r.svg, /<circle cx="300" cy="200" r="50"/);
  assert.deepEqual(out, ['Привіт']);
});

test('errors point at their place in UTF-16', async () => {
  const yu = await loadYu(wasm, quiet);
  const r = yu.run('бал = 1\nскаж(бал)', 'uk', [1, 0]);
  assert.equal(r.ok, false);
  assert.equal(r.error?.from, 8);
  assert.equal(r.error?.to, 12);
  assert.match(r.error?.text ?? '', /^Помилка в рядку 2: невідома назва «скаж»/);
});

test('ask gets its answer from the host', async () => {
  const out: string[] = [];
  const yu = await loadYu(wasm, { print: (line) => out.push(line), ask: () => 'Юрій' });
  yu.run('ім\'я = запитай("Як тебе звати?")\nскажи("Привіт, " + ім\'я)', 'uk', [1, 0]);
  assert.deepEqual(out, ['Привіт, Юрій']);
});

test('deep recursion ends with the friendly error, not a crash', async () => {
  const yu = await loadYu(wasm, quiet);
  const r = yu.run('функція f(n):\n    поверни f(n + 1)\nf(1)', 'uk', [1, 0]);
  assert.match(r.error?.text ?? '', /забагато вкладених викликів/);
});

test('an endless loop stops at the step budget', async () => {
  const yu = await loadYu(wasm, quiet);
  const r = yu.run('поки так:\n    x = 1', 'en', [1, 0]);
  assert.match(r.error?.text ?? '', /the program runs too long/);
});

test('names come with help in both languages', async () => {
  const yu = await loadYu(wasm, quiet);
  const names = yu.names();
  assert.ok(names.keywords.includes('разів'));
  assert.equal(names.builtins.length, 24);
  assert.equal(names.builtins.find((b) => b.en === 'circle')?.sig_uk, 'коло(x, y, радіус)');
});

test('the library has every section, entry and colour', async () => {
  const yu = await loadYu(wasm, quiet);
  const library = yu.library();
  assert.deepEqual(
    library.sections.map((s) => s.id),
    ['basics', 'conditions', 'loops', 'functions', 'lists', 'drawing', 'turtle'],
  );
  assert.equal(library.sections.flatMap((s) => s.entries).length, 37);
  const circle = library.sections[5].entries.find((e) => e.en === 'circle');
  assert.equal(circle?.form_uk, 'коло(x, y, радіус)');
  assert.equal(circle?.ex_uk, 'колір("червоний")\nколо(300, 200, 80)');
  assert.equal(library.colors.length, 12);
  assert.deepEqual(library.colors[0], { uk: 'червоний', en: 'red', hex: '#ef4444' });
});
