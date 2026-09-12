// Yu Studio: examples, running, output, the picture, links, autosave, themes and languages.
import '@fontsource-variable/inter';
import '@fontsource-variable/jetbrains-mono';
import './theme.css';
import type { EditorView } from '@codemirror/view';
import { loadYu, type YuLang, type YuResult } from './bridge.ts';
import { createEditor, setCode, showError } from './editor.ts';
import { EXAMPLES, text, type Key } from './i18n.ts';
import { Runner } from './runner.ts';
import { decode, encode, tokenFromHash } from './share.ts';

const files = import.meta.glob('../../examples/*.yu', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;
const example = (id: string) => files[`../../examples/${id}.yu`] ?? '';

const $ = <T extends HTMLElement = HTMLElement>(id: string) => document.getElementById(id) as T;
const runButton = $<HTMLButtonElement>('run');
const examples = $<HTMLSelectElement>('examples');
const output = $('output');
const picture = $('picture');
const status = $('status');
const toast = $('toast');

/** localStorage may be missing or throw (private windows); the page works without it. */
const store = {
  get(key: string): string | null {
    try {
      return localStorage.getItem(key);
    } catch {
      return null;
    }
  },
  set(key: string, value: string): void {
    try {
      localStorage.setItem(key, value);
    } catch {
      // Not remembered this time.
    }
  },
};

let lang: YuLang = store.get('yu-lang') === 'en' ? 'en' : 'uk';
let lastSvg = '';
let editor: EditorView;
let toastTimer = 0;
let saveTimer = 0;
const runner = new Runner();
const t = (key: Key) => text(lang, key);

function setTheme(theme: string | null): void {
  const next = theme === 'light' ? 'light' : 'dark';
  document.documentElement.dataset.theme = next;
  store.set('yu-theme', next);
}

function setRunLabel(): void {
  runButton.textContent = t(runner.running ? 'stop' : 'run');
  runButton.classList.toggle('stop', runner.running);
  status.classList.toggle('busy', runner.running);
}

function showEmpty(key: 'empty' | 'nothingDrawn'): void {
  const note = document.createElement('p');
  note.className = 'empty';
  note.textContent = t(key);
  picture.replaceChildren(note);
}

function applyLang(): void {
  document.documentElement.lang = lang;
  for (const el of document.querySelectorAll<HTMLElement>('[data-text]')) {
    el.textContent = t(el.dataset.text as Key);
  }
  const langButton = $<HTMLButtonElement>('lang');
  langButton.textContent = t('otherLang');
  langButton.title = t('otherLangName');
  const themeButton = $('theme');
  themeButton.title = t('theme');
  themeButton.setAttribute('aria-label', t('theme'));
  runButton.title = t('runHint');
  $('save-svg').title = t('saveTitle');
  $('save-png').title = t('saveTitle');
  examples.setAttribute('aria-label', t('examples'));
  const placeholder = new Option(t('examples'), '', true, true);
  placeholder.disabled = true;
  examples.replaceChildren(placeholder, ...EXAMPLES.map((e) => new Option(e[lang], e.id)));
  setRunLabel();
  if (!lastSvg) showEmpty('empty');
}

function line(content: string, kind?: 'error' | 'note'): void {
  const row = document.createElement('div');
  row.textContent = content;
  if (kind) row.className = kind;
  output.append(row);
  output.scrollTop = output.scrollHeight;
}

function notify(key: Key): void {
  toast.textContent = t(key);
  toast.hidden = false;
  clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => {
    toast.hidden = true;
  }, 2200);
}

function askInline(prompt: string): Promise<string | null> {
  if (!runner.canAsk) {
    line(prompt);
    line(t('noInput'), 'note');
    return Promise.resolve(null);
  }
  const row = document.createElement('div');
  row.className = 'ask';
  const question = document.createElement('span');
  question.textContent = prompt;
  const input = document.createElement('input');
  input.setAttribute('aria-label', t('answer'));
  row.append(question, input);
  output.append(row);
  input.focus();
  return new Promise((resolve) => {
    input.addEventListener('keydown', (event) => {
      if (event.key !== 'Enter') return;
      input.disabled = true;
      resolve(input.value);
    });
  });
}

function finish(result: YuResult): void {
  if (result.drew) {
    picture.innerHTML = result.svg;
    lastSvg = result.svg;
  } else {
    lastSvg = '';
    showEmpty('nothingDrawn');
  }
  if (result.error) {
    line(result.error.text, 'error');
    showError(editor, result.error);
  }
  status.textContent = result.ok ? t('done') : '';
  setRunLabel();
}

function run(): void {
  if (runner.running) {
    runner.stop();
    for (const input of output.querySelectorAll('input')) input.disabled = true;
    status.textContent = t('stopped');
    setRunLabel();
    return;
  }
  output.replaceChildren();
  status.textContent = '';
  showError(editor, null);
  runner.run(editor.state.doc.toString(), lang, {
    print: (content) => line(content),
    ask: askInline,
    done: finish,
    crash: (message) => {
      line(`${t('crash')}: ${message}`, 'error');
      setRunLabel();
    },
  });
  setRunLabel();
}

async function share(): Promise<void> {
  const token = await encode(editor.state.doc.toString());
  history.replaceState(null, '', `#code=${token}`);
  try {
    await navigator.clipboard.writeText(location.href);
    notify('copied');
  } catch {
    notify('linkInBar');
  }
}

function save(blob: Blob, ext: string): void {
  const link = document.createElement('a');
  link.href = URL.createObjectURL(blob);
  link.download = `${t('fileName')}.${ext}`;
  link.click();
  setTimeout(() => URL.revokeObjectURL(link.href), 1000);
}

/** The finished picture (no animation) at twice the canvas size. */
async function savePng(): Promise<void> {
  const still = lastSvg.replace(/<style>[\s\S]*?<\/style>/, '');
  const [, width = '600', height = '400'] = /width="(\d+)" height="(\d+)"/.exec(still) ?? [];
  const url = URL.createObjectURL(new Blob([still], { type: 'image/svg+xml' }));
  const image = new Image();
  image.src = url;
  await image.decode();
  const canvas = document.createElement('canvas');
  canvas.width = Number(width) * 2;
  canvas.height = Number(height) * 2;
  canvas.getContext('2d')?.drawImage(image, 0, 0, canvas.width, canvas.height);
  URL.revokeObjectURL(url);
  canvas.toBlob((blob) => {
    if (blob) save(blob, 'png');
  }, 'image/png');
}

function remember(code: string): void {
  clearTimeout(saveTimer);
  saveTimer = window.setTimeout(() => store.set('yu-code', code), 400);
}

async function firstProgram(): Promise<{ code: string; broken: boolean }> {
  const fallback = store.get('yu-code') ?? example('star');
  const token = tokenFromHash(location.hash);
  if (!token) return { code: fallback, broken: false };
  try {
    return { code: await decode(token), broken: false };
  } catch {
    return { code: fallback, broken: true };
  }
}

async function start(): Promise<void> {
  setTheme(store.get('yu-theme'));
  applyLang();
  line(t('loading'), 'note');
  const [yu, first] = await Promise.all([
    fetch('/yu_wasm.wasm')
      .then((response) => response.arrayBuffer())
      .then((bytes) => loadYu(bytes, { print: () => {}, ask: () => null })),
    firstProgram(),
  ]);
  editor = createEditor($('editor'), {
    doc: first.code,
    names: yu.names(),
    uiLang: () => lang,
    onRun: run,
    onChange: remember,
  });
  output.replaceChildren();
  if (first.broken) notify('badLink');

  runButton.addEventListener('click', run);
  $('share').addEventListener('click', () => void share());
  $('lang').addEventListener('click', () => {
    lang = lang === 'uk' ? 'en' : 'uk';
    store.set('yu-lang', lang);
    applyLang();
  });
  $('theme').addEventListener('click', () => {
    setTheme(document.documentElement.dataset.theme === 'dark' ? 'light' : 'dark');
  });
  examples.addEventListener('change', () => {
    setCode(editor, example(examples.value));
    showError(editor, null);
    examples.value = '';
    editor.focus();
  });
  $('save-svg').addEventListener('click', () => {
    if (lastSvg) save(new Blob([lastSvg], { type: 'image/svg+xml' }), 'svg');
    else notify('noPicture');
  });
  $('save-png').addEventListener('click', () => {
    if (lastSvg) void savePng();
    else notify('noPicture');
  });
  document.addEventListener('keydown', (event) => {
    if ((event.ctrlKey || event.metaKey) && event.key === 'Enter' && !editor.hasFocus) run();
  });
  editor.focus();
}

start().catch((error: unknown) => {
  output.replaceChildren();
  line(`${t('crash')}: ${String(error)}`, 'error');
});
