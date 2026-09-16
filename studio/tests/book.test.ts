import { test } from 'node:test';
import assert from 'node:assert/strict';
import { chapterList, escapeHtml, headings, highlightToHtml, slug, title } from '../src/book-tools.ts';
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
