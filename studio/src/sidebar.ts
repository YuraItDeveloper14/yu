// The activity bar and the sidebar: Examples, the Library, or nothing.
export type View = 'examples' | 'library';

const VIEWS: View[] = ['examples', 'library'];

export interface SidebarParts {
  root: HTMLElement;
  title: HTMLElement;
  buttons: Record<View, HTMLButtonElement>;
  panes: Record<View, HTMLElement>;
  close: HTMLButtonElement;
}

/** On narrow screens the sidebar is a drawer over the editor. */
export const narrow = (): boolean => window.matchMedia('(max-width: 900px)').matches;

export class Sidebar {
  view: View | null = null;
  /** The view Ctrl+B brings back. */
  private last: View = 'examples';
  private readonly parts: SidebarParts;
  private readonly titles: () => Record<View, string>;
  private readonly onChange: (view: View | null) => void;

  constructor(
    parts: SidebarParts,
    titles: () => Record<View, string>,
    onChange: (view: View | null) => void,
  ) {
    this.parts = parts;
    this.titles = titles;
    this.onChange = onChange;
    for (const view of VIEWS) {
      parts.buttons[view].addEventListener('click', () => this.set(this.view === view ? null : view));
    }
    parts.close.addEventListener('click', () => this.set(null));
    parts.root.addEventListener('keydown', (event) => {
      if (event.key === 'Escape' && narrow()) this.set(null);
    });
  }

  /** Shows `view`; null hides the sidebar. */
  set(view: View | null): void {
    this.view = view;
    if (view) this.last = view;
    this.parts.root.hidden = view === null;
    for (const v of VIEWS) {
      this.parts.panes[v].hidden = v !== view;
      this.parts.buttons[v].setAttribute('aria-pressed', String(v === view));
    }
    this.label();
    this.onChange(view);
  }

  /** Ctrl+B: hides the sidebar, or brings back the last view. */
  toggle(): void {
    this.set(this.view ? null : this.last);
  }

  /** The heading, in the interface language. */
  label(): void {
    if (this.view) this.parts.title.textContent = this.titles()[this.view];
  }
}
