// Yu Studio: the open program, runs, the menu, the sidebar, the output and the status bar.
import '@fontsource-variable/inter';
import '@fontsource-variable/jetbrains-mono';
import './theme.css';
import type { EditorView } from '@codemirror/view';
import { loadYu, type Yu, type YuLang, type YuResult } from './bridge.ts';
import { bindKeys, keyLabel, type Command } from './commands.ts';
import {
  createEditor,
  editorCommands,
  insertAtCursor,
  insertExample,
  setCode,
  setEditorLang,
  showError,
} from './editor.ts';
import {
  TooBig,
  download,
  openFile,
  pictureName,
  saveFile,
  saveFileAs,
  type FileHandle,
} from './files.ts';
import { EXAMPLES, text, type Key } from './i18n.ts';
import { LibraryView, type TryResult } from './library.ts';
import { MenuBar, type MenuDef } from './menu.ts';
import { Output, tabAfterRun } from './output.ts';
import { Runner } from './runner.ts';
import { decode, encode, tokenFromHash } from './share.ts';
import { Sidebar, narrow } from './sidebar.ts';

const files = import.meta.glob('../../examples/*.yu', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>;
const example = (id: string) => files[`../../examples/${id}.yu`] ?? '';

const $ = <T extends HTMLElement = HTMLElement>(id: string) => document.getElementById(id) as T;
const runButton = $<HTMLButtonElement>('run');
const toast = $('toast');
const mac = /Mac|iPhone|iPad/.test(navigator.platform);

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

type State = 'loading' | 'idle' | 'running' | 'done' | 'stopped' | 'failed';
const STATE_WORDS: Record<State, Key | null> = {
  loading: 'loading',
  idle: null,
  running: 'running',
  done: 'done',
  stopped: 'stopped',
  failed: 'failed',
};

let lang: YuLang = store.get('yu-lang') === 'en' ? 'en' : 'uk';
let state: State = 'loading';
let editor: EditorView;
let yu: Yu;
let library: LibraryView | null = null;
let lastSvg = '';
let cursor = { line: 1, column: 1 };
let toastTimer = 0;
let saveTimer = 0;
/** What the library's examples print while they run on the page's thread. */
let printed: string[] = [];
const runner = new Runner();
const t = (key: Key, values?: Record<string, string | number>) => text(lang, key, values);

/** The open program: its file name, its file on disk, the example it came from, the text last saved. */
const doc = {
  name: 'star.yu',
  handle: null as FileHandle | null,
  example: null as string | null,
  saved: null as string | null,
};
const code = () => editor.state.doc.toString();
const dirty = () => doc.saved !== code();

const output = new Output({
  tabs: { picture: $<HTMLButtonElement>('tab-picture'), terminal: $<HTMLButtonElement>('tab-terminal') },
  panels: { picture: $('picture'), terminal: $('terminal') },
  pictureActions: $('picture-actions'),
});

const sidebar = new Sidebar(
  {
    root: $('sidebar'),
    title: $('side-title'),
    buttons: {
      examples: $<HTMLButtonElement>('act-examples'),
      library: $<HTMLButtonElement>('act-library'),
    },
    panes: { examples: $('examples-pane'), library: $('library-pane') },
    close: $<HTMLButtonElement>('side-close'),
  },
  () => ({ examples: t('examples'), library: t('library') }),
  (view) => {
    // A phone's closed drawer shouldn't close the sidebar on the computer.
    if (!narrow()) store.set('yu-side', view ?? 'none');
  },
);

const commands: Command[] = [
  { id: 'new', label: () => 'newFile', run: newFile },
  { id: 'open', label: () => 'open', keys: 'Mod-O', run: () => void openFromDisk() },
  { id: 'save', label: () => 'save', keys: 'Mod-S', run: () => void save(false) },
  { id: 'save-as', label: () => 'saveAs', keys: 'Mod-Shift-S', run: () => void save(true) },
  { id: 'share', label: () => 'shareLink', run: () => void share() },
  { id: 'save-svg', label: () => 'savePictureSvg', run: () => savePicture('svg') },
  { id: 'save-png', label: () => 'savePicturePng', run: () => savePicture('png') },
  { id: 'undo', label: () => 'undo', keys: 'Mod-Z', editorKeys: true, run: () => edit(editorCommands.undo) },
  { id: 'redo', label: () => 'redo', keys: 'Mod-Y', editorKeys: true, run: () => edit(editorCommands.redo) },
  { id: 'find', label: () => 'find', keys: 'Mod-F', editorKeys: true, run: () => edit(editorCommands.find) },
  { id: 'comment', label: () => 'comment', keys: 'Mod-/', editorKeys: true, run: () => edit(editorCommands.comment) },
  { id: 'select-all', label: () => 'selectAll', keys: 'Mod-A', editorKeys: true, run: () => edit(editorCommands.selectAll) },
  { id: 'examples', label: () => 'examples', run: () => sidebar.set('examples') },
  { id: 'library', label: () => 'library', run: openLibrary },
  { id: 'sidebar', label: () => 'sidebar', keys: 'Mod-B', run: () => sidebar.toggle() },
  { id: 'picture', label: () => 'picture', run: () => output.show('picture') },
  { id: 'terminal', label: () => 'terminal', run: () => output.show('terminal') },
  {
    id: 'theme',
    label: () => (theme() === 'dark' ? 'themeLight' : 'themeDark'),
    run: () => setTheme(theme() === 'dark' ? 'light' : 'dark'),
  },
  { id: 'lang', label: () => 'otherLang', run: () => setLang(lang === 'uk' ? 'en' : 'uk') },
  { id: 'run', label: () => (runner.running ? 'runStop' : 'run'), keys: 'Mod-Enter', run },
  { id: 'library-words', label: () => 'libraryWords', run: openLibrary },
  {
    id: 'book',
    label: () => 'book',
    run: () => void window.open(`/book/${lang}/01-start/`, '_blank', 'noopener'),
  },
  {
    id: 'github',
    label: () => 'github',
    run: () => void window.open('https://github.com/YuraItDeveloper14/yu', '_blank', 'noopener'),
  },
];

const menus: MenuDef[] = [
  { title: 'menuFile', items: ['new', 'open', 'save', 'save-as', '-', 'share', 'save-svg', 'save-png'] },
  { title: 'menuEdit', items: ['undo', 'redo', '-', 'find', '-', 'comment', 'select-all'] },
  {
    title: 'menuView',
    items: ['examples', 'library', 'sidebar', '-', 'picture', 'terminal', '-', 'theme', 'lang'],
  },
  { title: 'menuRun', items: ['run'] },
  { title: 'menuHelp', items: ['library-words', 'book', 'github'] },
];

const menuBar = new MenuBar($('menus'), menus, {
  commands,
  t: (key) => t(key),
  keyLabel: (keys) => keyLabel(keys, mac),
});

function theme(): 'dark' | 'light' {
  return document.documentElement.dataset.theme === 'light' ? 'light' : 'dark';
}

function setTheme(next: string | null): void {
  const value = next === 'light' ? 'light' : 'dark';
  document.documentElement.dataset.theme = value;
  store.set('yu-theme', value);
}

function setLang(next: YuLang): void {
  lang = next;
  store.set('yu-lang', lang);
  applyLang();
}

/** Every word on the page, in the interface language. */
function applyLang(): void {
  document.documentElement.lang = lang;
  for (const el of document.querySelectorAll<HTMLElement>('[data-text]')) {
    el.textContent = t(el.dataset.text as Key);
  }
  const labels: [string, Key][] = [
    ['act-examples', 'examples'],
    ['act-library', 'library'],
    ['side-close', 'close'],
    ['save-svg', 'savePictureSvg'],
    ['save-png', 'savePicturePng'],
    ['st-theme', 'switchTheme'],
  ];
  for (const [id, key] of labels) {
    $(id).title = t(key);
    $(id).setAttribute('aria-label', t(key));
  }
  $('st-lang').textContent = t('langCode');
  $('st-lang').title = t('switchLang');
  $('st-version').title = t('releases');
  menuBar.render();
  sidebar.label();
  renderExamples();
  library?.render();
  if (editor) setEditorLang(editor, lang);
  if (!lastSvg) output.picture(null, t(state === 'loading' || state === 'idle' ? 'empty' : 'nothingDrawn'));
  showState();
  showCursor();
  refreshTitle();
}

function setState(next: State): void {
  state = next;
  showState();
}

/** The Run button and the status bar follow the run. */
function showState(): void {
  const running = runner.running;
  $('run-label').textContent = t(running ? 'stop' : 'run');
  runButton.classList.toggle('stop', running);
  runButton.setAttribute('aria-label', t(running ? 'stop' : 'run'));
  runButton.title = `${t(running ? 'runStop' : 'run')} (${keyLabel('Mod-Enter', mac)})`;
  const word = STATE_WORDS[state];
  const status = $('st-state');
  status.textContent = word ? t(word) : '';
  status.classList.toggle('busy', state === 'running');
}

function showCursor(): void {
  $('st-pos').textContent = t('position', cursor);
}

/** The file tab, the status bar and the browser tab show the file name, with ● for unsaved changes. */
function refreshTitle(): void {
  const changed = editor ? dirty() : false;
  $('file-name').textContent = doc.name;
  $('file-dot').hidden = !changed;
  $('st-file').textContent = changed ? `${doc.name} ●` : doc.name;
  document.title = `${changed ? '● ' : ''}${doc.name} — Yu Studio`;
  for (const item of document.querySelectorAll<HTMLElement>('[data-example]')) {
    item.setAttribute('aria-current', String(item.dataset.example === doc.example));
  }
}

/** Autosave: the text, the name and whether it is saved survive a reload. */
function remember(): void {
  clearTimeout(saveTimer);
  saveTimer = window.setTimeout(() => {
    store.set('yu-code', code());
    store.set('yu-file', doc.name);
    store.set('yu-dirty', dirty() ? '1' : '0');
  }, 400);
}

function say(message: string): void {
  toast.textContent = message;
  toast.hidden = false;
  clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => {
    toast.hidden = true;
  }, 2600);
}

function notify(key: Key): void {
  say(t(key));
}

function reason(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/** Puts a program into the editor as the open file. */
function openDoc(
  name: string,
  program: string,
  from: { handle?: FileHandle | null; example?: string | null } = {},
): void {
  doc.name = name;
  doc.handle = from.handle ?? null;
  doc.example = from.example ?? null;
  doc.saved = program;
  setCode(editor, program);
  showError(editor, null);
  refreshTitle();
  remember();
}

/** True when nothing unsaved would be lost, or the user agrees to lose it. */
function mayReplace(): boolean {
  return !dirty() || window.confirm(t('discard', { name: doc.name }));
}

function newFile(): void {
  if (mayReplace()) openDoc(t('untitled'), '');
  editor.focus();
}

function openExample(id: string): void {
  if (!mayReplace()) return;
  openDoc(`${id}.yu`, example(id), { example: id });
  if (narrow()) sidebar.set(null);
  editor.focus();
}

/** The examples in the sidebar; the open one is marked. */
function renderExamples(): void {
  const items = EXAMPLES.map((e) => {
    const item = document.createElement('button');
    item.type = 'button';
    item.className = 'example';
    item.dataset.example = e.id;
    const title = document.createElement('span');
    title.textContent = e[lang];
    const file = document.createElement('span');
    file.className = 'file';
    file.textContent = `${e.id}.yu`;
    item.append(title, file);
    item.addEventListener('click', () => openExample(e.id));
    return item;
  });
  $('examples-pane').replaceChildren(...items);
}

async function openFromDisk(): Promise<void> {
  if (!mayReplace()) return;
  try {
    const file = await openFile();
    if (file) openDoc(file.name, file.text, { handle: file.handle });
  } catch (error) {
    say(error instanceof TooBig ? t('tooBig') : t('openFailed', { reason: reason(error) }));
  }
  editor.focus();
}

async function save(as: boolean): Promise<void> {
  const program = code();
  try {
    const saved = as
      ? await saveFileAs(program, doc.name)
      : await saveFile(program, doc.name, doc.handle);
    if (!saved) return;
    doc.name = saved.name;
    doc.handle = saved.handle;
    doc.example = null;
    doc.saved = program;
    refreshTitle();
    remember();
  } catch (error) {
    say(t('saveFailed', { reason: reason(error) }));
  }
}

async function share(): Promise<void> {
  const token = await encode(code());
  history.replaceState(null, '', `#code=${token}`);
  try {
    await navigator.clipboard.writeText(location.href);
    notify('copied');
  } catch {
    notify('linkInBar');
  }
}

function savePicture(ext: 'svg' | 'png'): void {
  if (!lastSvg) return notify('noPicture');
  if (ext === 'svg') download(new Blob([lastSvg], { type: 'image/svg+xml' }), pictureName(doc.name, 'svg'));
  else void savePng();
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
    if (blob) download(blob, pictureName(doc.name, 'png'));
  }, 'image/png');
}

function openLibrary(): void {
  sidebar.set('library');
  document.getElementById('library-search')?.focus();
}

/** Runs one of the editor's own commands from the menu and gives the editor the focus back. */
function edit(command: (view: EditorView) => boolean): void {
  command(editor);
  editor.focus();
}

/** Runs a library example on the page's thread; examples are short and the core's tests prove they end. */
function tryExample(program: string): TryResult {
  printed = [];
  const result = yu.run(program, lang, [1, 0]);
  return { lines: printed, svg: result.drew ? result.svg : null, error: result.error?.text ?? null };
}

function run(): void {
  if (runner.running) {
    runner.stop();
    output.closeInputs();
    output.print(t('stopped'), 'note');
    output.show('terminal');
    setState('stopped');
    return;
  }
  const file = /\s/.test(doc.name) ? `"${doc.name}"` : doc.name;
  output.begin(`yu ${file}`);
  showError(editor, null);
  runner.run(code(), lang, {
    print: (line) => output.print(line),
    ask: (prompt) => output.ask(prompt, runner.canAsk, { answer: t('answer'), noInput: t('noInput') }),
    done: finish,
    crash: (message) => {
      output.print(`${t('crash')}: ${message}`, 'error');
      output.show('terminal');
      setState('failed');
    },
  });
  setState('running');
}

function finish(result: YuResult): void {
  // The tab opens first, so a new picture behind the Terminal tab gets its dot.
  output.show(tabAfterRun(result));
  lastSvg = result.drew ? result.svg : '';
  output.picture(result.drew ? result.svg : null, t('nothingDrawn'));
  if (result.error) {
    output.print(result.error.text, 'error');
    showError(editor, result.error);
  } else {
    output.print(t('done'), 'note');
  }
  setState(result.ok ? 'done' : 'failed');
}

interface First {
  name: string;
  code: string;
  saved: boolean;
  example: string | null;
  broken: boolean;
}

/** What the editor starts with: a shared link, else the autosaved program, else the star. */
async function firstProgram(): Promise<First> {
  const token = tokenFromHash(location.hash);
  if (token) {
    try {
      return { name: t('sharedName'), code: await decode(token), saved: true, example: null, broken: false };
    } catch {
      // A broken link: the last program opens instead.
    }
  }
  const broken = token !== null;
  const last = store.get('yu-code');
  if (last === null) return { name: 'star.yu', code: example('star'), saved: true, example: 'star', broken };
  return {
    name: store.get('yu-file') || t('sharedName'),
    code: last,
    saved: store.get('yu-dirty') !== '1',
    example: EXAMPLES.find((e) => example(e.id) === last)?.id ?? null,
    broken,
  };
}

async function start(): Promise<void> {
  setTheme(store.get('yu-theme'));
  applyLang();
  const [module, first] = await Promise.all([
    fetch('/yu_wasm.wasm')
      .then((response) => response.arrayBuffer())
      .then((bytes) =>
        loadYu(bytes, {
          print: (line) => void printed.push(line),
          ask: (prompt) => {
            const answer = t('sampleAnswer');
            printed.push(`${prompt} ${answer}`);
            return answer;
          },
        }),
      ),
    firstProgram(),
  ]);
  yu = module;
  const names = yu.names();
  $('st-version').textContent = `Yu ${names.version}`;
  doc.name = first.name;
  doc.example = first.example;
  doc.saved = first.saved ? first.code : null;
  editor = createEditor($('editor'), {
    doc: first.code,
    names,
    uiLang: () => lang,
    onRun: run,
    onChange: () => {
      refreshTitle();
      remember();
    },
    onCursor: (line, column) => {
      cursor = { line, column };
      showCursor();
    },
  });
  library = new LibraryView($('library-pane'), yu.library(), names, {
    lang: () => lang,
    words: () => ({
      search: t('searchLabel'),
      searchExample: t('searchPlaceholder'),
      insert: t('insert'),
      try: t('try'),
      nothingFound: t('nothingFound'),
      colours: t('colours'),
      coloursText: t('coloursText'),
      picture: t('picture'),
    }),
    insertExample: (program) => {
      insertExample(editor, program);
      if (narrow()) sidebar.set(null);
    },
    insertText: (words) => insertAtCursor(editor, words),
    tryExample,
  });
  const side = store.get('yu-side');
  sidebar.set(narrow() || side === 'none' ? null : side === 'library' ? 'library' : 'examples');
  state = 'idle';
  applyLang();
  if (first.broken) notify('badLink');
  bindKeys(commands, mac);
  runButton.addEventListener('click', run);
  $('share').addEventListener('click', () => void share());
  $('save-svg').addEventListener('click', () => savePicture('svg'));
  $('save-png').addEventListener('click', () => savePicture('png'));
  $('st-lang').addEventListener('click', () => setLang(lang === 'uk' ? 'en' : 'uk'));
  $('st-theme').addEventListener('click', () => setTheme(theme() === 'dark' ? 'light' : 'dark'));
  editor.focus();
}

start().catch((error: unknown) => {
  output.print(`${t('crash')}: ${String(error)}`, 'error');
  output.show('terminal');
  setState('failed');
});
