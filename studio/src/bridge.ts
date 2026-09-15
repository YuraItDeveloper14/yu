// Loads the Yu core (yu_wasm.wasm) and talks to it through its small hand-written ABI.

export type YuLang = 'uk' | 'en';

export interface YuError {
  text: string;
  /** UTF-16 offsets, as the editor counts. */
  from: number;
  to: number;
}

export interface YuResult {
  ok: boolean;
  drew: boolean;
  svg: string;
  error?: YuError;
}

export interface YuBuiltin {
  uk: string;
  en: string;
  sig_uk: string;
  sig_en: string;
  doc_uk: string;
  doc_en: string;
}

export interface YuNames {
  keywords: string[];
  builtins: YuBuiltin[];
}

/** One word of the library; `ex_uk` and `ex_en` are its example program. */
export interface YuEntry {
  uk: string;
  en: string;
  form_uk: string;
  form_en: string;
  text_uk: string;
  text_en: string;
  ex_uk: string;
  ex_en: string;
}

export interface YuSection {
  id: string;
  uk: string;
  en: string;
  entries: YuEntry[];
}

export interface YuColor {
  uk: string;
  en: string;
  hex: string;
}

export interface YuLibrary {
  sections: YuSection[];
  colors: YuColor[];
}

/** What a running program needs from its host. */
export interface Hooks {
  print(line: string): void;
  /** The answer to запитай, or null when there is none. */
  ask(prompt: string): string | null;
}

export interface Yu {
  run(code: string, lang: YuLang, seed: [number, number]): YuResult;
  names(): YuNames;
  library(): YuLibrary;
}

interface Exports {
  memory: WebAssembly.Memory;
  alloc(len: number): number;
  dealloc(ptr: number, len: number): void;
  run(ptr: number, len: number, lang: number, seedLo: number, seedHi: number): number;
  names(): number;
  library(): number;
}

export async function loadYu(wasm: BufferSource, hooks: Hooks): Promise<Yu> {
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
  let exports: Exports | undefined;
  // A fresh view every time: the memory's buffer is replaced when it grows.
  const bytes = () => new Uint8Array((exports as Exports).memory.buffer);
  const read = (ptr: number, len: number) => decoder.decode(bytes().subarray(ptr, ptr + len));
  const env = {
    host_print(ptr: number, len: number): void {
      hooks.print(read(ptr, len));
    },
    host_ask(promptPtr: number, promptLen: number, outPtr: number, outCap: number): number {
      const answer = hooks.ask(read(promptPtr, promptLen));
      if (answer === null) return -1;
      const encoded = encoder.encode(answer).subarray(0, outCap);
      bytes().set(encoded, outPtr);
      return encoded.length;
    },
  };
  const { instance } = await WebAssembly.instantiate(wasm, { env });
  const ex = instance.exports as unknown as Exports;
  exports = ex;

  /** Reads a length-prefixed result and frees it. */
  const take = (ptr: number): string => {
    const len = new DataView(ex.memory.buffer).getUint32(ptr, true);
    const text = read(ptr + 4, len);
    ex.dealloc(ptr, len + 4);
    return text;
  };

  return {
    run(code, lang, [seedLo, seedHi]) {
      const src = encoder.encode(code);
      const ptr = ex.alloc(src.length);
      bytes().set(src, ptr);
      try {
        const result = ex.run(ptr, src.length, lang === 'en' ? 1 : 0, seedLo, seedHi);
        return JSON.parse(take(result)) as YuResult;
      } finally {
        ex.dealloc(ptr, src.length);
      }
    },
    names() {
      return JSON.parse(take(ex.names())) as YuNames;
    },
    library() {
      return JSON.parse(take(ex.library())) as YuLibrary;
    },
  };
}
