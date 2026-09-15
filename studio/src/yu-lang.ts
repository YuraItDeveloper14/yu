// Yu for CodeMirror: highlighting, indentation after ':' and completion of the core's words.
import { StreamLanguage, indentService, type StreamParser } from '@codemirror/language';
import type {
  Completion,
  CompletionContext,
  CompletionResult,
  CompletionSource,
} from '@codemirror/autocomplete';
import type { EditorView } from '@codemirror/view';
import type { YuLang, YuNames } from './bridge.ts';

type State = Record<string, never>;

const NAME = /^[\p{L}_][\p{L}\p{N}_]*(?:['’ʼ]\p{L}[\p{L}\p{N}_]*)*/u;
const NUMBER = /^\d[\d_]*(?:\.\d+)?/;
const OPERATOR = /^(?:==|!=|<=|>=|[+\-*/%=<>])/;
const WORD = /[\p{L}_][\p{L}\p{N}_'’ʼ]*/u;

/** The indent of a new line after `prev`: one unit deeper when `prev` opens a block. */
export function indentFor(prev: string, prevIndent: number, unit: number): number {
  const code = prev.replace(/#.*$/, '').trimEnd();
  return code.endsWith(':') ? prevIndent + unit : prevIndent;
}

/**
 * How a library example goes into the program at a line: it takes the place of a blank line,
 * and otherwise follows the line, at the indentation a new line would get there.
 */
export function placeExample(
  line: string,
  example: string,
  unit: number,
): { replace: boolean; text: string } {
  const indent = line.length - line.trimStart().length;
  const blank = line.trim() === '';
  const pad = ' '.repeat(blank ? indent : indentFor(line, indent, unit));
  const body = example
    .split('\n')
    .map((row) => (row ? pad + row : row))
    .join('\n');
  return { replace: blank, text: blank ? body : `\n${body}` };
}

export function yuParser(
  keywords: ReadonlySet<string>,
  builtins: ReadonlySet<string>,
): StreamParser<State> {
  return {
    name: 'yu',
    startState: () => ({}),
    token(stream) {
      if (stream.eatSpace()) return null;
      if (stream.peek() === '#') {
        stream.skipToEnd();
        return 'comment';
      }
      if (stream.eat('"')) {
        let escaped = false;
        for (let ch = stream.next(); ch; ch = stream.next()) {
          if (ch === '"' && !escaped) break;
          escaped = ch === '\\' && !escaped;
        }
        return 'string';
      }
      if (stream.match(NUMBER)) return 'number';
      if (stream.match(NAME)) {
        const word = stream.current();
        if (keywords.has(word)) return 'keyword';
        if (builtins.has(word)) return 'variableName.standard';
        return 'variableName';
      }
      if (stream.match(OPERATOR)) return 'operator';
      stream.next();
      return null;
    },
    languageData: { commentTokens: { line: '#' } },
  };
}

export function yuLanguage(names: YuNames): StreamLanguage<State> {
  const builtins = names.builtins.flatMap((b) => [b.uk, b.en]);
  return StreamLanguage.define(yuParser(new Set(names.keywords), new Set(builtins)));
}

/** A new line after one that ends in ':' goes one indent unit deeper. */
export const yuIndent = indentService.of((cx, pos) => {
  if (pos === 0) return 0;
  const prev = cx.lineAt(pos - 1);
  return indentFor(prev.text, cx.lineIndent(prev.from), cx.unit);
});

/** A built-in in the list: its brackets follow the name, its description is in the UI language. */
function builtin(label: string, signature: string, info: string): Completion {
  return {
    label,
    type: 'function',
    detail: signature.slice(label.length),
    info,
    apply: (view: EditorView, _completion: Completion, from: number, to: number) => {
      view.dispatch({
        changes: { from, to, insert: `${label}()` },
        selection: { anchor: from + label.length + 1 },
      });
    },
  };
}

export function yuCompletions(names: YuNames, uiLang: () => YuLang): CompletionSource {
  const keywords: Completion[] = names.keywords.map((label) => ({ label, type: 'keyword' }));
  return (ctx: CompletionContext): CompletionResult | null => {
    const word = ctx.matchBefore(WORD);
    if (!word && !ctx.explicit) return null;
    const uk = uiLang() === 'uk';
    const builtins = names.builtins.flatMap((b) => {
      const info = uk ? b.doc_uk : b.doc_en;
      return [builtin(b.uk, b.sig_uk, info), builtin(b.en, b.sig_en, info)];
    });
    return {
      from: word ? word.from : ctx.pos,
      options: [...builtins, ...keywords],
      validFor: /^[\p{L}\p{N}_'’ʼ]*$/u,
    };
  };
}
