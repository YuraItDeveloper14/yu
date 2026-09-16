import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chapterList, description, escapeHtml, headings, highlightToHtml, slug, title } from '../src/book-tools.ts';
import { yuParser } from '../src/yu-lang.ts';

const chapter = [
  '# Цикли',
  '',
  'Текст розділу.',
  '',
  '## Повтори',
  '',
  '```yu',
  '# Це коментар, а не заголовок',
  'повтори 2 рази:',
  '    скажи("Крок")',
  '```',
  '',
  '## Спробуй сам',
  '',
].join('\n');

test('a heading becomes a short anchor', () => {
  assert.equal(slug('Спробуй сам'), 'спробуй-сам');
  assert.equal(slug('For … from … to'), 'for-from-to');
  assert.equal(slug('Так, ні, нічого'), 'так-ні-нічого');
});

test('the title and the headings skip what is inside code', () => {
  assert.equal(title(chapter), 'Цикли');
  assert.deepEqual(headings(chapter), [
    { text: 'Повтори', anchor: 'повтори' },
    { text: 'Спробуй сам', anchor: 'спробуй-сам' },
  ]);
});

test('the description is the first paragraph as plain text, in whole sentences', () => {
  const start = [
    '# Перша програма',
    '',
    'Yu працює просто в браузері: відкрий [yu-lang.vercel.app](https://yu-lang.vercel.app) — і можна',
    'писати. Встановлювати нічого не треба.',
    '',
    'Другий абзац сюди не потрапляє.',
  ].join('\n');
  assert.equal(
    description(start),
    'Yu працює просто в браузері: відкрий yu-lang.vercel.app — і можна писати. Встановлювати нічого не треба.',
  );
  const words = ['<!-- generated -->', '', '# Усі слова', '', 'Кожне слово: `скажи(…)` і **решта**. Далі.'].join('\n');
  assert.equal(description(words), 'Кожне слово: скажи(…) і решта. Далі.');
});

test('a description stops before 160 characters, at a sentence or else at a word', () => {
  const sentence = 'Слово '.repeat(20).trim() + '.';
  const two = `# Т\n\n${sentence} ${sentence}`;
  assert.equal(description(two), sentence);
  const long = `# Т\n\n${'довгеслово '.repeat(30).trim()}.`;
  const cut = description(long);
  assert.ok(cut.length <= 160, cut);
  assert.ok(cut.endsWith('довгеслово…'), cut);
});

test('chapters keep their file order and know their address', () => {
  const list = chapterList(
    [
      { file: '04-loops', markdown: chapter },
      { file: '01-start', markdown: '# Перша програма\n' },
    ],
    'uk',
  );
  assert.deepEqual(
    list.map((c) => [c.file, c.id, c.title, c.url]),
    [
      ['01-start', 'start', 'Перша програма', '/book/uk/01-start/'],
      ['04-loops', 'loops', 'Цикли', '/book/uk/04-loops/'],
    ],
  );
});

test('code turns into coloured, escaped HTML', () => {
  const parser = yuParser(new Set(['повтори', 'разів']), new Set(['скажи']));
  const html = highlightToHtml('скажи("<&>")', parser);
  assert.match(html, /<span class="tok-variableName-standard">скажи<\/span>/);
  assert.match(html, /&lt;&amp;&gt;/);
  assert.equal(escapeHtml('a<b>"c"&d'), 'a&lt;b&gt;&quot;c&quot;&amp;d');
});
