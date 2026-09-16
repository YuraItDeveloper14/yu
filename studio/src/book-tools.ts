// Pieces the book's generator and its pages share: anchors, chapter lists and coloured code.
import { StringStream, type StreamParser } from '@codemirror/language';

export interface Chapter {
  /** The file name without its number, e.g. `loops`. */
  id: string;
  /** The file name, e.g. `04-loops`. */
  file: string;
  title: string;
  /** Where the page lives, e.g. `/book/uk/04-loops/`. */
  url: string;
  headings: { text: string; anchor: string }[];
}

export function escapeHtml(text: string): string {
  const named: Record<string, string> = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' };
  return text.replace(/[&<>"]/g, (character) => named[character] ?? character);
}

/** A heading's anchor: lower case, letters and digits kept, everything else a dash. */
export function slug(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, '-')
    .replace(/^-+|-+$/g, '');
}

/** The lines of `markdown` that are not inside a fenced block. */
function prose(markdown: string): string[] {
  const out: string[] = [];
  let inside = false;
  for (const line of markdown.split('\n')) {
    if (line.startsWith('```')) {
      inside = !inside;
      continue;
    }
    if (!inside) out.push(line);
  }
  return out;
}

/** The chapter's `# ` heading; a `#` comment inside an example is not one. */
export function title(markdown: string): string {
  const line = prose(markdown).find((row) => row.startsWith('# '));
  return line ? line.slice(2).trim() : '';
}

/** Every `## ` heading of the chapter, with its anchor. */
export function headings(markdown: string): { text: string; anchor: string }[] {
  return prose(markdown)
    .filter((line) => line.startsWith('## '))
    .map((line) => {
      const text = line.slice(3).trim();
      return { text, anchor: slug(text) };
    });
}

/** The page's description: the first paragraph as plain text, in whole sentences, at most 160 characters. */
export function description(markdown: string): string {
  const lines: string[] = [];
  for (const line of prose(markdown)) {
    const text = line.trim();
    const skip = !text || text.startsWith('#') || text.startsWith('<');
    if (skip && lines.length) break;
    if (!skip) lines.push(text);
  }
  const plain = lines
    .join(' ')
    .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
    .replace(/`([^`]*)`/g, '$1')
    .replace(/\*\*([^*]+)\*\*|\*([^*]+)\*/g, '$1$2');
  let out = '';
  for (const sentence of plain.split(/(?<=[.!?…])\s+/)) {
    const next = out ? `${out} ${sentence}` : sentence;
    if (next.length > 160) break;
    out = next;
  }
  return out || `${plain.slice(0, 159).replace(/\s+\S*$/, '')}…`;
}

/** The chapters of one language, in file order. */
export function chapterList(
  files: { file: string; markdown: string }[],
  lang: string,
): Chapter[] {
  return files
    .slice()
    .sort((a, b) => a.file.localeCompare(b.file))
    .map(({ file, markdown }) => ({
      id: file.replace(/^\d+-/, ''),
      file,
      title: title(markdown),
      url: `/book/${lang}/${file}/`,
      headings: headings(markdown),
    }));
}

/** Code coloured with the editor's own tokenizer, as HTML. */
export function highlightToHtml(
  code: string,
  parser: StreamParser<Record<string, never>>,
): string {
  let out = '';
  code.split('\n').forEach((line, index) => {
    if (index > 0) out += '\n';
    const stream = new StringStream(line, 4, 4);
    while (!stream.eol()) {
      const style = parser.token(stream, {});
      const piece = escapeHtml(stream.current());
      out += style ? `<span class="tok-${style.replace('.', '-')}">${piece}</span>` : piece;
      stream.start = stream.pos;
    }
  });
  return out;
}
