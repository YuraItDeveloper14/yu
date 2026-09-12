// Messages between the page and the worker that runs Yu.
import type { YuLang, YuResult } from './bridge.ts';

export type ToWorker =
  | { type: 'init'; shared: SharedArrayBuffer | null }
  | { type: 'run'; code: string; lang: YuLang; seed: [number, number] };

export type FromWorker =
  | { type: 'print'; line: string }
  | { type: 'ask'; prompt: string }
  | { type: 'done'; result: YuResult }
  | { type: 'crash'; message: string };

/** Bytes an answer may take; the module on the other side has the same room. */
export const ANSWER_CAP = 4096;
/** Shared memory: Int32 state at byte 0, Int32 length at byte 4, answer bytes from byte 8. */
export const ANSWERED = 1;
export const NO_ANSWER = 2;
