// The CodeMirror editor: Yu language, colours from the theme, Ctrl+Enter, the error underline.
import { EditorState } from '@codemirror/state';
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
  toggleComment,
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
import { tags } from '@lezer/highlight';
import type { YuError, YuLang, YuNames } from './bridge.ts';
import { yuCompletions, yuIndent, yuLanguage } from './yu-lang.ts';

export interface EditorOptions {
  doc: string;
  names: YuNames;
  uiLang: () => YuLang;
  onRun: () => void;
  onChange: (doc: string) => void;
}

const highlight = HighlightStyle.define([
  { tag: tags.keyword, color: 'var(--syntax-keyword)' },
  { tag: tags.standard(tags.variableName), color: 'var(--syntax-builtin)' },
  { tag: tags.string, color: 'var(--syntax-string)' },
  { tag: tags.number, color: 'var(--syntax-number)' },
  { tag: tags.comment, color: 'var(--syntax-comment)', fontStyle: 'italic' },
  { tag: tags.operator, color: 'var(--muted)' },
]);

const look = EditorView.theme({
  '&': { height: '100%', color: 'var(--text)', backgroundColor: 'var(--panel)', fontSize: '15px' },
  '.cm-scroller': { fontFamily: 'var(--font-code)', lineHeight: '1.65' },
  '.cm-content': { caretColor: 'var(--accent)', padding: '14px 0' },
  '.cm-gutters': { backgroundColor: 'var(--panel)', color: 'var(--muted)', border: 'none' },
  '.cm-activeLine, .cm-activeLineGutter': { backgroundColor: 'var(--active-line)' },
  '&.cm-focused': { outline: 'none' },
  '&.cm-focused .cm-cursor': { borderLeftColor: 'var(--accent)' },
  '&.cm-focused .cm-selectionBackground, .cm-selectionBackground': {
    backgroundColor: 'var(--selection)',
  },
  '.cm-tooltip': {
    backgroundColor: 'var(--panel)',
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
        keymap.of([
          { key: 'Mod-Enter', run },
          { key: 'Mod-/', run: toggleComment },
          ...closeBracketsKeymap,
          ...completionKeymap,
          ...historyKeymap,
          indentWithTab,
          ...defaultKeymap,
        ]),
        look,
        EditorView.contentAttributes.of({ 'aria-label': 'Yu', spellcheck: 'false' }),
        EditorView.updateListener.of((update) => {
          if (update.docChanged) opts.onChange(update.state.doc.toString());
        }),
      ],
    }),
  });
}

/** Replaces the whole program. */
export function setCode(view: EditorView, code: string): void {
  view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: code } });
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
