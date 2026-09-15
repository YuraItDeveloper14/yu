import { test } from 'node:test';
import assert from 'node:assert/strict';
import type { YuSection } from '../src/bridge.ts';
import { searchColors, searchLibrary } from '../src/library.ts';

const entry = (uk: string, en: string, text_uk = '', text_en = '') => ({
  uk,
  en,
  form_uk: `${uk}()`,
  form_en: `${en}()`,
  text_uk,
  text_en,
  ex_uk: '',
  ex_en: '',
});

const sections: YuSection[] = [
  {
    id: 'drawing',
    uk: 'Малювання',
    en: 'Drawing',
    entries: [entry('коло', 'circle', 'зафарбоване коло', 'a filled circle'), entry('лінія', 'line')],
  },
  {
    id: 'loops',
    uk: 'Цикли',
    en: 'Loops',
    entries: [entry('повтори … разів', 'repeat … times', 'пиши раз, рази чи разів', 'times may be left out')],
  },
];

test('search finds either spelling, ignores case and drops empty sections', () => {
  assert.deepEqual(
    searchLibrary(sections, 'КОЛО', 'uk').map((s) => s.entries.map((e) => e.en)),
    [['circle']],
  );
  assert.deepEqual(searchLibrary(sections, 'circle', 'uk').map((s) => s.id), ['drawing']);
  assert.deepEqual(searchLibrary(sections, '  ', 'uk'), sections);
  assert.deepEqual(searchLibrary(sections, 'щось', 'uk'), []);
});

test('search reads the form and the text in the interface language', () => {
  assert.deepEqual(searchLibrary(sections, 'рази', 'uk').map((s) => s.id), ['loops']);
  assert.deepEqual(searchLibrary(sections, 'рази', 'en'), []);
  assert.deepEqual(searchLibrary(sections, 'left out', 'en').map((s) => s.id), ['loops']);
});

test('colours match either name, and random is a colour of its own', () => {
  const colors = [
    { uk: 'червоний', en: 'red', hex: '#ef4444' },
    { uk: 'синій', en: 'blue', hex: '#0057b7' },
  ];
  assert.deepEqual(searchColors(colors, 'СИН').map((c) => c.en), ['blue']);
  assert.deepEqual(searchColors(colors, '').map((c) => c.en), ['red', 'blue', 'random']);
  assert.deepEqual(searchColors(colors, 'випад').map((c) => c.en), ['random']);
});
