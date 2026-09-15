// The menu bar: menus built from the command table, used by mouse or keyboard like a desktop editor's.
import type { Command } from './commands.ts';
import type { Key } from './i18n.ts';

export interface MenuDef {
  title: Key;
  /** Command ids in order; '-' draws a separator. */
  items: string[];
}

export interface MenuDeps {
  commands: Command[];
  t(key: Key): string;
  /** How the menu writes a command's keys. */
  keyLabel(keys: string): string;
}

export class MenuBar {
  private readonly bar: HTMLElement;
  private readonly menus: MenuDef[];
  private readonly deps: MenuDeps;
  private readonly popup: HTMLElement;
  private titles: HTMLButtonElement[] = [];
  /** The open menu's index, or -1. */
  private open = -1;
  /** Where the focus goes back when the menu closes. */
  private returnFocus: HTMLElement | null = null;

  constructor(bar: HTMLElement, menus: MenuDef[], deps: MenuDeps) {
    this.bar = bar;
    this.menus = menus;
    this.deps = deps;
    this.popup = document.createElement('div');
    this.popup.className = 'menu';
    this.popup.setAttribute('role', 'menu');
    this.popup.hidden = true;
    document.body.append(this.popup);
    bar.addEventListener('keydown', (event) => this.onKey(event));
    this.popup.addEventListener('keydown', (event) => this.onKey(event));
    document.addEventListener('pointerdown', (event) => {
      const target = event.target as Node;
      if (this.open >= 0 && !bar.contains(target) && !this.popup.contains(target)) this.close(false);
    });
    window.addEventListener('resize', () => this.close(false));
    this.render();
  }

  /** Draws the menu titles again, e.g. in another language. */
  render(): void {
    this.titles = this.menus.map((menu, index) => {
      const title = document.createElement('button');
      title.type = 'button';
      title.className = 'menu-title';
      title.setAttribute('role', 'menuitem');
      title.setAttribute('aria-haspopup', 'menu');
      title.setAttribute('aria-expanded', 'false');
      title.textContent = this.deps.t(menu.title);
      title.addEventListener('pointerdown', () => {
        if (this.open < 0) this.returnFocus = document.activeElement as HTMLElement | null;
      });
      title.addEventListener('click', () =>
        this.open === index ? this.close(true) : this.show(index, false),
      );
      title.addEventListener('pointerenter', () => {
        if (this.open >= 0 && this.open !== index) this.show(index, false);
      });
      return title;
    });
    this.bar.replaceChildren(...this.titles);
    if (this.open >= 0) this.show(this.open, false);
  }

  /** Closes the open menu; with `restore` the focus goes back where it was. */
  close(restore: boolean): void {
    if (this.open < 0) return;
    this.mark(this.open, false);
    this.open = -1;
    this.popup.hidden = true;
    if (restore) this.returnFocus?.focus();
    this.returnFocus = null;
  }

  private show(index: number, focusFirst: boolean): void {
    if (this.open < 0 && !this.returnFocus) {
      this.returnFocus = document.activeElement as HTMLElement | null;
    }
    if (this.open >= 0) this.mark(this.open, false);
    this.open = index;
    this.mark(index, true);
    this.popup.replaceChildren(...this.menus[index].items.map((id) => this.item(id)));
    this.popup.hidden = false;
    const box = this.titles[index].getBoundingClientRect();
    const left = Math.min(box.left, window.innerWidth - this.popup.offsetWidth - 8);
    this.popup.style.left = `${Math.max(8, left)}px`;
    this.popup.style.top = `${box.bottom + 4}px`;
    if (focusFirst) this.items()[0]?.focus();
  }

  private mark(index: number, open: boolean): void {
    this.titles[index]?.setAttribute('aria-expanded', String(open));
    this.titles[index]?.classList.toggle('open', open);
  }

  private item(id: string): HTMLElement {
    if (id === '-') {
      const line = document.createElement('div');
      line.setAttribute('role', 'separator');
      return line;
    }
    const command = this.deps.commands.find((c) => c.id === id);
    if (!command) throw new Error(`No command "${id}"`);
    const item = document.createElement('button');
    item.type = 'button';
    item.tabIndex = -1;
    item.setAttribute('role', 'menuitem');
    item.dataset.command = id;
    const label = document.createElement('span');
    label.textContent = this.deps.t(command.label());
    item.append(label);
    if (command.keys) {
      const keys = document.createElement('kbd');
      keys.textContent = this.deps.keyLabel(command.keys);
      item.append(keys);
    }
    item.addEventListener('click', () => {
      this.close(true);
      command.run();
    });
    return item;
  }

  private items(): HTMLButtonElement[] {
    return [...this.popup.querySelectorAll<HTMLButtonElement>('[role="menuitem"]')];
  }

  /** Arrow keys, Escape and Tab, as in a desktop menu bar. */
  private onKey(event: KeyboardEvent): void {
    const onTitle = this.titles.indexOf(event.target as HTMLButtonElement);
    const index = onTitle >= 0 ? onTitle : this.open;
    const items = this.items();
    const at = items.indexOf(document.activeElement as HTMLButtonElement);
    switch (event.key) {
      case 'Escape':
        if (this.open < 0) return;
        this.close(true);
        break;
      case 'ArrowDown':
        if (index < 0) return;
        if (this.open !== index) this.show(index, true);
        else items[(at + 1) % items.length]?.focus();
        break;
      case 'ArrowUp':
        if (this.open < 0) return;
        items[at <= 0 ? items.length - 1 : at - 1]?.focus();
        break;
      case 'ArrowRight':
      case 'ArrowLeft': {
        if (index < 0) return;
        const step = event.key === 'ArrowRight' ? 1 : -1;
        const next = (index + step + this.titles.length) % this.titles.length;
        if (this.open >= 0) this.show(next, true);
        else this.titles[next].focus();
        break;
      }
      case 'Tab':
        this.close(false);
        return;
      default:
        return;
    }
    event.preventDefault();
    event.stopPropagation();
  }
}
