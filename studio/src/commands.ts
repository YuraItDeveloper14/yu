// Commands and their keys: one table that the menu, the key handler and the key labels all read.
import type { Key } from './i18n.ts';

export interface Command {
  id: string;
  /** Its name in the menu; asked for each time the menu opens, because some names follow the state. */
  label(): Key;
  /** Keys like 'Mod-Shift-S'; Mod is Ctrl, or ⌘ on a Mac. */
  keys?: string;
  /** The editor handles these keys itself; the menu only shows them. */
  editorKeys?: boolean;
  run(): void;
}

/** The parts of a key event a shortcut depends on. */
export type KeyPress = Pick<KeyboardEvent, 'code' | 'ctrlKey' | 'metaKey' | 'shiftKey' | 'altKey'>;

const CODES: Record<string, string> = { Enter: 'Enter', '/': 'Slash', '`': 'Backquote' };
const MAC_NAMES: Record<string, string> = { Mod: '⌘', Shift: '⇧', Alt: '⌥' };

/** The physical key a shortcut ends with: 'S' is KeyS on every keyboard layout. */
function code(key: string): string {
  if (/^[A-Z]$/.test(key)) return `Key${key}`;
  if (/^[0-9]$/.test(key)) return `Digit${key}`;
  return CODES[key] ?? key;
}

/** Whether `event` is the shortcut `keys`, matched by physical key so any layout works. */
export function matches(keys: string, event: KeyPress, mac: boolean): boolean {
  const parts = keys.split('-');
  const key = parts.pop() ?? '';
  return (
    event.code === code(key) &&
    (mac ? event.metaKey : event.ctrlKey) === parts.includes('Mod') &&
    !(mac ? event.ctrlKey : event.metaKey) &&
    event.shiftKey === parts.includes('Shift') &&
    event.altKey === parts.includes('Alt')
  );
}

/** How the menu writes `keys`: Ctrl+Shift+S, or ⌘⇧S on a Mac. */
export function keyLabel(keys: string, mac: boolean): string {
  const parts = keys.split('-');
  const key = parts.pop() ?? '';
  if (mac) return parts.map((part) => MAC_NAMES[part] ?? part).join('') + key;
  return [...parts.map((part) => (part === 'Mod' ? 'Ctrl' : part)), key].join('+');
}

/** Runs the command whose keys were pressed, unless the editor already handled them. */
export function bindKeys(commands: Command[], mac: boolean): void {
  window.addEventListener('keydown', (event) => {
    if (event.defaultPrevented || event.repeat) return;
    const command = commands.find(
      (c) => c.keys !== undefined && !c.editorKeys && matches(c.keys, event, mac),
    );
    if (!command) return;
    event.preventDefault();
    command.run();
  });
}
