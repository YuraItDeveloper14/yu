// Runs Yu programs off the page's thread; запитай waits on shared memory for the answer.
import { loadYu, type Yu } from './bridge.ts';
import { ANSWERED, type FromWorker, type ToWorker } from './protocol.ts';

const scope = self as unknown as {
  postMessage(message: FromWorker): void;
  onmessage: ((event: MessageEvent<ToWorker>) => void) | null;
};
const send = (message: FromWorker) => scope.postMessage(message);

let shared: SharedArrayBuffer | null = null;

function ask(prompt: string): string | null {
  if (!shared) {
    // No shared memory: the page still shows the question and says why no answer can come.
    send({ type: 'ask', prompt });
    return null;
  }
  const cells = new Int32Array(shared, 0, 2);
  Atomics.store(cells, 0, 0);
  send({ type: 'ask', prompt });
  Atomics.wait(cells, 0, 0);
  if (Atomics.load(cells, 0) !== ANSWERED) return null;
  const length = Atomics.load(cells, 1);
  // slice() copies out of shared memory: TextDecoder refuses shared views.
  return new TextDecoder().decode(new Uint8Array(shared, 8, length).slice());
}

const yu: Promise<Yu> = fetch('/yu_wasm.wasm')
  .then((response) => response.arrayBuffer())
  .then((bytes) => loadYu(bytes, { print: (line) => send({ type: 'print', line }), ask }));

scope.onmessage = async (event) => {
  const message = event.data;
  if (message.type === 'init') {
    shared = message.shared;
    return;
  }
  try {
    const result = (await yu).run(message.code, message.lang, message.seed);
    send({ type: 'done', result });
  } catch (error) {
    send({ type: 'crash', message: String(error) });
  }
};
