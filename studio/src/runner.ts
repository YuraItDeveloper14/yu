// The page side of the worker: starts runs, answers запитай, stops endless programs.
import type { YuLang, YuResult } from './bridge.ts';
import {
  ANSWERED,
  ANSWER_CAP,
  NO_ANSWER,
  type FromWorker,
  type ToWorker,
} from './protocol.ts';

export interface RunEvents {
  print(line: string): void;
  ask(prompt: string): Promise<string | null>;
  done(result: YuResult): void;
  crash(message: string): void;
}

export class Runner {
  /** Whether запитай can wait for an answer: the page must be cross-origin isolated. */
  readonly canAsk: boolean;
  private readonly shared: SharedArrayBuffer | null;
  private worker: Worker;
  private events: RunEvents | null = null;

  constructor() {
    this.canAsk = typeof SharedArrayBuffer !== 'undefined' && crossOriginIsolated;
    this.shared = this.canAsk ? new SharedArrayBuffer(8 + ANSWER_CAP) : null;
    this.worker = this.spawn();
  }

  get running(): boolean {
    return this.events !== null;
  }

  run(code: string, lang: YuLang, events: RunEvents): void {
    this.events = events;
    const seed = crypto.getRandomValues(new Uint32Array(2));
    this.post({ type: 'run', code, lang, seed: [seed[0], seed[1]] });
  }

  /** Ends the current program at once; the next run gets a fresh worker. */
  stop(): void {
    this.events = null;
    this.restart();
  }

  private spawn(): Worker {
    const worker = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module' });
    worker.onmessage = (event: MessageEvent<FromWorker>) => void this.receive(event.data);
    worker.postMessage({ type: 'init', shared: this.shared } as ToWorker);
    return worker;
  }

  private post(message: ToWorker): void {
    this.worker.postMessage(message);
  }

  private restart(): void {
    this.worker.terminate();
    this.worker = this.spawn();
  }

  private async receive(message: FromWorker): Promise<void> {
    const events = this.events;
    if (!events) return;
    if (message.type === 'print') {
      events.print(message.line);
    } else if (message.type === 'ask') {
      const answer = await events.ask(message.prompt);
      // A stopped program's question must not answer the next program's.
      if (this.events === events) this.answer(answer);
    } else {
      this.events = null;
      if (message.type === 'done') {
        events.done(message.result);
      } else {
        this.restart();
        events.crash(message.message);
      }
    }
  }

  private answer(text: string | null): void {
    if (!this.shared) return;
    const cells = new Int32Array(this.shared, 0, 2);
    if (text !== null) {
      const bytes = new TextEncoder().encode(text).subarray(0, ANSWER_CAP);
      new Uint8Array(this.shared, 8, bytes.length).set(bytes);
      Atomics.store(cells, 1, bytes.length);
    }
    Atomics.store(cells, 0, text === null ? NO_ANSWER : ANSWERED);
    Atomics.notify(cells, 0);
  }
}
