// The one output: Picture and Terminal tabs over one panel, and the rule that picks the tab.
export type Tab = 'picture' | 'terminal';

const TABS: Tab[] = ['picture', 'terminal'];

/** The tab a finished run shows: Picture after a drawing, Terminal after an error or plain text. */
export function tabAfterRun(result: { ok: boolean; drew: boolean }): Tab {
  return result.ok && result.drew ? 'picture' : 'terminal';
}

export interface OutputParts {
  tabs: Record<Tab, HTMLButtonElement>;
  panels: Record<Tab, HTMLElement>;
  /** The SVG and PNG buttons: they belong next to the picture. */
  pictureActions: HTMLElement;
}

export class Output {
  current: Tab = 'picture';
  private readonly parts: OutputParts;

  constructor(parts: OutputParts) {
    this.parts = parts;
    for (const tab of TABS) {
      parts.tabs[tab].addEventListener('click', () => this.show(tab));
      parts.tabs[tab].addEventListener('keydown', (event) => {
        if (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight') return;
        const other: Tab = tab === 'picture' ? 'terminal' : 'picture';
        this.show(other);
        parts.tabs[other].focus();
      });
    }
  }

  /** Opens `tab` and clears its dot. */
  show(tab: Tab): void {
    this.current = tab;
    for (const t of TABS) {
      const selected = t === tab;
      this.parts.tabs[t].setAttribute('aria-selected', String(selected));
      this.parts.tabs[t].tabIndex = selected ? 0 : -1;
      this.parts.panels[t].hidden = !selected;
    }
    this.parts.pictureActions.hidden = tab !== 'picture';
    this.mark(tab, false);
  }

  /** A clean terminal for a new run, headed by the command that runs it. */
  begin(command: string): void {
    const head = document.createElement('div');
    head.className = 'cmd';
    const sign = document.createElement('span');
    sign.textContent = '›';
    head.append(sign, ` ${command}`);
    this.parts.panels.terminal.replaceChildren(head);
    this.mark('terminal', false);
    this.mark('picture', false);
  }

  /** One line in the terminal; printed lines and errors put a dot on a hidden Terminal tab. */
  print(content: string, kind?: 'error' | 'note'): void {
    const row = document.createElement('div');
    row.textContent = content;
    if (kind) row.className = kind;
    this.append(row);
    if (kind !== 'note') this.mark('terminal', true);
  }

  /** The question of запитай with an answer field; resolves with the answer, or null without one. */
  ask(prompt: string, canAsk: boolean, words: { answer: string; noInput: string }): Promise<string | null> {
    this.show('terminal');
    if (!canAsk) {
      this.print(prompt);
      this.print(words.noInput, 'note');
      return Promise.resolve(null);
    }
    const row = document.createElement('div');
    row.className = 'ask';
    const question = document.createElement('span');
    question.textContent = prompt;
    const input = document.createElement('input');
    input.setAttribute('aria-label', words.answer);
    row.append(question, input);
    this.append(row);
    input.focus();
    return new Promise((resolve) => {
      input.addEventListener('keydown', (event) => {
        if (event.key !== 'Enter') return;
        input.disabled = true;
        resolve(input.value);
      });
    });
  }

  /** A stopped program takes no more answers. */
  closeInputs(): void {
    for (const input of this.parts.panels.terminal.querySelectorAll('input')) input.disabled = true;
  }

  /** The picture, or a note when there is none; a new picture puts a dot on a hidden Picture tab. */
  picture(svg: string | null, note: string): void {
    if (svg) {
      this.parts.panels.picture.innerHTML = svg;
      this.mark('picture', true);
      return;
    }
    const empty = document.createElement('p');
    empty.className = 'empty';
    empty.textContent = note;
    this.parts.panels.picture.replaceChildren(empty);
  }

  private append(row: HTMLElement): void {
    const terminal = this.parts.panels.terminal;
    terminal.append(row);
    terminal.scrollTop = terminal.scrollHeight;
  }

  /** The dot on a tab with something new; the open tab never has one. */
  private mark(tab: Tab, fresh: boolean): void {
    const dot = this.parts.tabs[tab].querySelector<HTMLElement>('.dot');
    if (dot) dot.hidden = !fresh || tab === this.current;
  }
}
