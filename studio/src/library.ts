// The library view: every word of Yu with what it does and an example to insert or try.
import { StringStream, type StreamParser } from '@codemirror/language';
import type { YuColor, YuEntry, YuLang, YuLibrary, YuNames, YuSection } from './bridge.ts';
import { yuParser } from './yu-lang.ts';

/** What Try shows: the printed lines, the picture and the error, if any. */
export interface TryResult {
  lines: string[];
  svg: string | null;
  error: string | null;
}

/** The library's own interface words, in the interface language. */
export interface LibraryWords {
  search: string;
  searchExample: string;
  insert: string;
  try: string;
  nothingFound: string;
  colours: string;
  coloursText: string;
  picture: string;
}

export interface LibraryDeps {
  lang(): YuLang;
  words(): LibraryWords;
  /** Puts an example into the program. */
  insertExample(code: string): void;
  /** Types text at the cursor. */
  insertText(text: string): void;
  /** Runs an example on the page's thread. */
  tryExample(code: string): TryResult;
}

/** `випадковий` is not in the palette but works wherever a colour does. */
const RANDOM: YuColor = { uk: 'випадковий', en: 'random', hex: '' };

const pick = (lang: YuLang, uk: string, en: string) => (lang === 'uk' ? uk : en);

/** Entries matching `query` in either name, or in the form or text of `lang`; empty sections drop out. */
export function searchLibrary(sections: YuSection[], query: string, lang: YuLang): YuSection[] {
  const q = query.trim().toLowerCase();
  if (!q) return sections;
  const hit = (e: YuEntry) =>
    [e.uk, e.en, pick(lang, e.form_uk, e.form_en), pick(lang, e.text_uk, e.text_en)].some((field) =>
      field.toLowerCase().includes(q),
    );
  return sections
    .map((section) => ({ ...section, entries: section.entries.filter(hit) }))
    .filter((section) => section.entries.length > 0);
}

/** Colours whose names match `query`, with `випадковий` / `random` last. */
export function searchColors(colors: YuColor[], query: string): YuColor[] {
  const q = query.trim().toLowerCase();
  const all = [...colors, RANDOM];
  return q ? all.filter((c) => c.uk.includes(q) || c.en.includes(q)) : all;
}

function button(label: string, onClick: () => void): HTMLButtonElement {
  const b = document.createElement('button');
  b.type = 'button';
  b.className = 'ghost small';
  b.textContent = label;
  b.addEventListener('click', onClick);
  return b;
}

export class LibraryView {
  private readonly library: YuLibrary;
  private readonly deps: LibraryDeps;
  private readonly parser: StreamParser<Record<string, never>>;
  private readonly search: HTMLInputElement;
  private readonly results: HTMLElement;

  constructor(root: HTMLElement, library: YuLibrary, names: YuNames, deps: LibraryDeps) {
    this.library = library;
    this.deps = deps;
    this.parser = yuParser(
      new Set(names.keywords),
      new Set(names.builtins.flatMap((b) => [b.uk, b.en])),
    );
    this.search = document.createElement('input');
    this.search.type = 'search';
    this.search.id = 'library-search';
    this.search.className = 'search';
    this.search.addEventListener('input', () => this.render());
    this.results = document.createElement('div');
    this.results.className = 'results';
    root.replaceChildren(this.search, this.results);
    this.render();
  }

  /** Draws the entries again: after typing in the search field, or in another language. */
  render(): void {
    const lang = this.deps.lang();
    const words = this.deps.words();
    this.search.placeholder = words.searchExample;
    this.search.setAttribute('aria-label', words.search);
    const sections = searchLibrary(this.library.sections, this.search.value, lang);
    const colors = searchColors(this.library.colors, this.search.value);
    const parts = sections.map((s) =>
      this.section(pick(lang, s.uk, s.en), s.entries.map((e) => this.entry(e))),
    );
    if (colors.length) parts.push(this.section(words.colours, [this.palette(colors)]));
    if (!parts.length) {
      const none = document.createElement('p');
      none.className = 'none';
      none.textContent = words.nothingFound;
      parts.push(none);
    }
    this.results.replaceChildren(...parts);
  }

  private section(title: string, children: HTMLElement[]): HTMLElement {
    const box = document.createElement('section');
    const head = document.createElement('h3');
    head.textContent = title;
    box.append(head, ...children);
    return box;
  }

  private entry(e: YuEntry): HTMLElement {
    const lang = this.deps.lang();
    const words = this.deps.words();
    const [main, other] = lang === 'uk' ? [e.uk, e.en] : [e.en, e.uk];
    const box = document.createElement('details');
    box.className = 'entry';
    box.dataset.word = e.uk;
    const summary = document.createElement('summary');
    const word = document.createElement('code');
    word.textContent = main;
    const alt = document.createElement('span');
    alt.className = 'alt';
    alt.textContent = other;
    summary.append(word, alt);
    const form = document.createElement('code');
    form.className = 'form';
    form.textContent = pick(lang, e.form_uk, e.form_en);
    const text = document.createElement('p');
    text.textContent = pick(lang, e.text_uk, e.text_en);
    const example = pick(lang, e.ex_uk, e.ex_en);
    const code = document.createElement('pre');
    code.className = 'snippet';
    code.append(this.colored(example));
    const result = document.createElement('div');
    result.className = 'result';
    result.hidden = true;
    const actions = document.createElement('div');
    actions.className = 'actions';
    actions.append(
      button(words.insert, () => this.deps.insertExample(example)),
      button(words.try, () => this.show(result, this.deps.tryExample(example))),
    );
    box.append(summary, form, text, code, actions, result);
    return box;
  }

  /** What an example printed and drew; the picture is an image, so its styles stay its own. */
  private show(box: HTMLElement, result: TryResult): void {
    const parts: HTMLElement[] = [];
    if (result.lines.length) {
      const out = document.createElement('pre');
      out.textContent = result.lines.join('\n');
      parts.push(out);
    }
    if (result.svg) {
      const image = document.createElement('img');
      image.src = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(result.svg)}`;
      image.alt = this.deps.words().picture;
      parts.push(image);
    }
    if (result.error) {
      const out = document.createElement('pre');
      out.className = 'error';
      out.textContent = result.error;
      parts.push(out);
    }
    box.replaceChildren(...parts);
    box.hidden = false;
  }

  private palette(colors: YuColor[]): HTMLElement {
    const lang = this.deps.lang();
    const box = document.createElement('div');
    box.className = 'palette';
    const note = document.createElement('p');
    note.textContent = this.deps.words().coloursText;
    const grid = document.createElement('div');
    grid.className = 'swatches';
    for (const color of colors) {
      const name = pick(lang, color.uk, color.en);
      const swatch = button('', () => this.deps.insertText(`"${name}"`));
      swatch.className = 'swatch';
      swatch.title = pick(lang, color.en, color.uk);
      const chip = document.createElement('span');
      chip.className = color.hex ? 'chip' : 'chip random';
      if (color.hex) chip.style.background = color.hex;
      const label = document.createElement('span');
      label.textContent = name;
      swatch.append(chip, label);
      grid.append(swatch);
    }
    box.append(note, grid);
    return box;
  }

  /** Example code coloured like the editor, with the same tokenizer. */
  private colored(code: string): DocumentFragment {
    const out = document.createDocumentFragment();
    code.split('\n').forEach((line, index) => {
      if (index > 0) out.append('\n');
      const stream = new StringStream(line, 4, 4);
      while (!stream.eol()) {
        const style = this.parser.token(stream, {});
        const piece = stream.current();
        if (style) {
          const span = document.createElement('span');
          span.className = `tok-${style.replace('.', '-')}`;
          span.textContent = piece;
          out.append(span);
        } else {
          out.append(piece);
        }
        stream.start = stream.pos;
      }
    });
    return out;
  }
}
