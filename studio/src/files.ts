// Programs on disk: the File System Access API where the browser has it (Chrome, Edge),
// otherwise a file input to open and a download to save.

/** The part of a file handle the Studio uses. */
export interface FileHandle {
  readonly name: string;
  getFile(): Promise<File>;
  createWritable(): Promise<{ write(data: string): Promise<void>; close(): Promise<void> }>;
}

interface Pickers {
  showOpenFilePicker?: (options: object) => Promise<FileHandle[]>;
  showSaveFilePicker?: (options: object) => Promise<FileHandle>;
}

/** A program opened from disk. */
export interface OpenedFile {
  name: string;
  text: string;
  /** Where Save writes it back; null when the browser can't write files. */
  handle: FileHandle | null;
}

/** Where a program was saved. */
export interface SavedFile {
  name: string;
  handle: FileHandle | null;
}

/** Programs are small; a bigger file is surely something else. */
export const MAX_BYTES = 1024 * 1024;

export class TooBig extends Error {}

const TYPES = [{ description: 'Yu', accept: { 'text/plain': ['.yu'] } }];

const pickers = (): Pickers => window as unknown as Pickers;

const cancelled = (error: unknown): boolean =>
  error instanceof DOMException && error.name === 'AbortError';

/** `star.yu` → `star`. */
export function baseName(name: string): string {
  const dot = name.lastIndexOf('.');
  return dot > 0 ? name.slice(0, dot) : name;
}

/** A picture is named after its program: `star.yu` → `star.svg`. */
export function pictureName(program: string, ext: 'svg' | 'png'): string {
  return `${baseName(program)}.${ext}`;
}

/** A file the user picks, or null when they cancel; throws TooBig for files over MAX_BYTES. */
export async function openFile(): Promise<OpenedFile | null> {
  const show = pickers().showOpenFilePicker;
  if (typeof show !== 'function') {
    const file = await chooseFile();
    return file ? read(file, null) : null;
  }
  try {
    const [handle] = await show.call(window, { types: TYPES });
    return await read(await handle.getFile(), handle);
  } catch (error) {
    if (cancelled(error)) return null;
    throw error;
  }
}

/** Writes `text` back to its file, or asks where when it has none; null when cancelled. */
export async function saveFile(
  text: string,
  name: string,
  handle: FileHandle | null,
): Promise<SavedFile | null> {
  if (!handle) return saveFileAs(text, name);
  await write(handle, text);
  return { name: handle.name, handle };
}

/** Asks where to save `text`, suggesting `name`; without the API the file is downloaded. */
export async function saveFileAs(text: string, name: string): Promise<SavedFile | null> {
  const show = pickers().showSaveFilePicker;
  if (typeof show !== 'function') {
    download(new Blob([text], { type: 'text/plain;charset=utf-8' }), name);
    return { name, handle: null };
  }
  let handle: FileHandle;
  try {
    handle = await show.call(window, { suggestedName: name, types: TYPES });
  } catch (error) {
    if (cancelled(error)) return null;
    throw error;
  }
  await write(handle, text);
  return { name: handle.name, handle };
}

/** Hands `blob` to the browser as a download called `name`. */
export function download(blob: Blob, name: string): void {
  const link = document.createElement('a');
  link.href = URL.createObjectURL(blob);
  link.download = name;
  link.click();
  setTimeout(() => URL.revokeObjectURL(link.href), 1000);
}

async function read(file: File, handle: FileHandle | null): Promise<OpenedFile> {
  if (file.size > MAX_BYTES) throw new TooBig(file.name);
  return { name: file.name, text: await file.text(), handle };
}

async function write(handle: FileHandle, text: string): Promise<void> {
  const stream = await handle.createWritable();
  await stream.write(text);
  await stream.close();
}

/** The file input for browsers without the API; null when its dialog is cancelled. */
function chooseFile(): Promise<File | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.yu,.txt,text/plain';
    input.hidden = true;
    const done = (file: File | null) => {
      input.remove();
      resolve(file);
    };
    input.addEventListener('change', () => done(input.files?.[0] ?? null));
    input.addEventListener('cancel', () => done(null));
    document.body.append(input);
    input.click();
  });
}
