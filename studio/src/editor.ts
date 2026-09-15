// The CodeMirror editor: Yu language, colours from the theme, keys, find and replace, library
// examples at the cursor, the cursor position and the error underline.
import { Compartment, EditorState } from '@codemirror/state';
import {
  EditorView,
  drawSelection,
  highlightActiveLine,
  highlightActiveLineGutter,
  keymap,
  lineNumbers,
} from '@codemirror/view';
import {
  defaultKeymap,
  history,
  historyKeymap,
  indentWithTab,
  redo,
  selectAll,
  toggleComment,
  undo,
} from '@codemirror/commands';
import {
  HighlightStyle,
  bracketMatching,
  indentOnInput,
  indentUnit,
  syntaxHighlighting,
} from '@codemirror/language';
import {
  autocompletion,
  closeBrackets,
  closeBracketsKeymap,
  completionKeymap,
} from '@codemirror/autocomplete';
import { setDiagnostics } from '@codemirror/lint';
import { openSearchPanel, search, searchKeymap } from '@codemirror/search';
import { tags } from '@lezer/highlight';
import type { YuError, YuLang, YuNames } from './bridge.ts';
import { placeExample, yuCompletions, yuIndent, yuLanguage } from './yu-lang.ts';

export interface EditorOptions {
  doc: string;
  names: YuNames;
  uiLang: () => YuLang;
  onRun: () => void;
  onChange: (doc: string) => void;
  /** The cursor's line and column, both counted from 1. */
  onCursor?: (line: number, column: number) => void;
}

/** The find panel's words in Ukrainian; English ones are CodeMirror's own. */
const UK_PHRASES: Record<string, string> = {
  Find: 'Знайти',
  Replace: 'Замінити',
  next: 'наступний',
  previous: 'попередній',
  all: 'усі',
  'match case': 'з урахуванням регістру',
  regexp: 'регулярний вираз',
  'by word': 'ціле слово',
  replace: 'замінити',
  'replace all': 'замінити всі',
  close: 'закрити',
};

const phrases = new Compartment();
const phrasesFor = (lang: YuLang) => EditorState.phrases.of(lang === 'uk' ? UK_PHRASES : {});

const highlight = HighlightStyle.define([
  { tag: tags.keyword, color: 'var(--syntax-keyword)' },
  { tag: tags.standard(tags.variableName), color: 'var(--syntax-builtin)' },
  { tag: tags.string, color: 'var(--syntax-string)' },
  { tag: tags.number, color: 'var(--syntax-number)' },
  { tag: tags.comment, color: 'var(--syntax-comment)', fontStyle: 'italic' },
  { tag: tags.operator, color: 'var(--muted)' },
]);

const look = EditorView.theme({
  '&': { height: '100%', color: 'var(--text)', backgroundColor: 'var(--surface)', fontSize: '15px' },
  '.cm-scroller': { fontFamily: 'var(--font-code)', lineHeight: '1.65' },
  '.cm-content': { caretColor: 'var(--accent-text)', padding: '14px 0' },
  '.cm-gutters': { backgroundColor: 'var(--surface)', color: 'var(--muted)', border: 'none' },
  '.cm-activeLine, .cm-activeLineGutter': { backgroundColor: 'var(--active-line)' },
  '&.cm-focused': { outline: 'none' },
  '&.cm-focused .cm-cursor': { borderLeftColor: 'var(--accent-text)' },
  '&.cm-focused .cm-selectionBackground, .cm-selectionBackground': {
    backgroundColor: 'var(--selection)',
  },
  '.cm-tooltip': {
    backgroundColor: 'var(--raised)',
    border: '1px solid var(--border)',
    borderRadius: '8px',
    color: 'var(--text)',
  },
  '.cm-tooltip-autocomplete ul li[aria-selected]': {
    backgroundColor: 'var(--accent-soft)',
    color: 'var(--text)',
  },
  '.cm-completionDetail': { color: 'var(--muted)', fontStyle: 'normal' },
  '.cm-completionInfo': { padding: '6px 10px' },
  '.cm-lintRange-error': {
    backgroundImage: 'none',
    textDecoration: 'underline wavy var(--error)',
    textUnderlineOffset: '4px',
  },
  '.cm-diagnostic-error': { borderLeftColor: 'var(--error)' },
  '.cm-panels': { backgroundColor: 'var(--raised)', color: 'var(--text)' },
  '.cm-panels-top': { borderBottom: '1px solid var(--border)' },
  '.cm-search': { fontFamily: 'var(--font-ui)', fontSize: '13px' },
  '.cm-textfield': {
    border: '1px solid var(--border)',
    borderRadius: '6px',
    backgroundColor: 'var(--surface)',
    color: 'var(--text)',
    padding: '3px 8px',
  },
  '.cm-button': {
    height: 'auto',
    backgroundImage: 'none',
    backgroundColor: 'transparent',
    border: '1px solid var(--border)',
    borderRadius: '6px',
    color: 'var(--text)',
    padding: '2px 10px',
  },
  '.cm-search [name=close]': {
    height: 'auto',
    border: 'none',
    background: 'none',
    color: 'var(--muted)',
    padding: '0 4px',
  },
  '.cm-searchMatch': { backgroundColor: 'rgba(255, 215, 0, 0.22)' },
  '.cm-searchMatch-selected': { backgroundColor: 'rgba(255, 215, 0, 0.45)' },
});

export function createEditor(parent: HTMLElement, opts: EditorOptions): EditorView {
  const run = () => {
    opts.onRun();
    return true;
  };
  return new EditorView({
    parent,
    state: EditorState.create({
      doc: opts.doc,
      extensions: [
        lineNumbers(),
        highlightActiveLine(),
        highlightActiveLineGutter(),
        history(),
        drawSelection(),
        indentOnInput(),
        bracketMatching(),
        closeBrackets(),
        indentUnit.of('    '),
        EditorState.tabSize.of(4),
        yuLanguage(opts.names),
        yuIndent,
        syntaxHighlighting(highlight),
        autocompletion({ override: [yuCompletions(opts.names, opts.uiLang)] }),
        search({ top: true }),
        phrases.of(phrasesFor(opts.uiLang())),
        keymap.of([
          { key: 'Mod-Enter', run },
          { key: 'Mod-/', run: toggleComment },
          ...closeBracketsKeymap,
          ...completionKeymap,
          ...searchKeymap,
          ...historyKeymap,
          indentWithTab,
          ...defaultKeymap,
        ]),
        look,
        EditorView.contentAttributes.of({ 'aria-label': 'Yu', spellcheck: 'false' }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) opts.onChange(update.state.doc.toString());
          if (update.docChanged || update.selectionSet) {
            const head = update.state.selection.main.head;
            const line = update.state.doc.lineAt(head);
            opts.onCursor?.(line.number, head - line.from + 1);
          }
        }),
      ],
    }),
  });
}

/** The editor's own commands, for the Edit menu. */
export const editorCommands = {
  undo,
  redo,
  find: openSearchPanel,
  comment: toggleComment,
  selectAll,
};

/** The find panel speaks the interface language. */
export function setEditorLang(view: EditorView, lang: YuLang): void {
  view.dispatch({ effects: phrases.reconfigure(phrasesFor(lang)) });
}

/** Replaces the whole program. */
export function setCode(view: EditorView, code: string): void {
  view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: code } });
}

/** Puts a library example into the program at the cursor's line (see placeExample). */
export function insertExample(view: EditorView, example: string): void {
  const line = view.state.doc.lineAt(view.state.selection.main.head);
  const { replace, text } = placeExample(line.text, example, 4);
  const from = replace ? line.from : line.to;
  view.dispatch({
    changes: { from, to: line.to, insert: text },
    selection: { anchor: from + text.length },
    scrollIntoView: true,
  });
  view.focus();
}

/** Types `text` at the cursor, as if the user had typed it. */
export function insertAtCursor(view: EditorView, text: string): void {
  view.dispatch(view.state.replaceSelection(text));
  view.focus();
}

/** Underlines the error from the last run, or removes the underline. */
export function showError(view: EditorView, error: YuError | null): void {
  const len = view.state.doc.length;
  const diagnostics = error
    ? [
        {
          from: Math.min(error.from, len),
          to: Math.min(Math.max(error.to, error.from + 1), len),
          severity: 'error' as const,
          message: error.text,
        },
      ]
    : [];
  view.dispatch(setDiagnostics(view.state, diagnostics));
}
