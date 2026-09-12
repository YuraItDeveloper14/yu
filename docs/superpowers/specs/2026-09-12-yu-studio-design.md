# Yu Studio — design (milestone 3)

Date: 2026-09-12 · Author: Yurii Dmytrenko · Status: approved for planning

Extends the [v1 language design](2026-09-11-yu-language-design.md) (its Playground section) and
the [drawing design](2026-09-11-yu-drawing-design.md). Where they differ, this document wins.

## Summary

Yu Studio is one web page for writing Yu: the editor on the left, the picture and the output on
the right. The Yu core runs in the browser as WebAssembly inside a Web Worker, so the page never
freezes and Stop always works. It is a static site deployed to Vercel as the project `yu-studio`.

## What the user sees

- **Themes.** Dark by default, in the palette of the GitHub profile card; a light theme is one
  click away. The choice is remembered.
- **Layout.** A top bar; under it the editor (about 55 % of the width) and a right column with the
  picture on top and the output below. Narrower than 900 px, the three stack: editor, picture,
  output.
- **Top bar.** The "Yu Studio" logo, an Examples menu, the UA/EN switch, the theme switch, Share,
  and the Run button, which becomes Stop while a program runs.
- **Editor** (CodeMirror 6):
  - highlighting of Yu in both languages: keywords, built-ins, text in quotes, numbers, comments;
  - completion of keywords and built-ins in both languages, each with its signature and a
    one-line description in the interface language; choosing a built-in inserts `name()` with
    the cursor inside the brackets;
  - Ctrl+Enter (Cmd+Enter on a Mac) runs; Enter after a line that ends in `:` indents by four
    spaces;
  - after a failed run the error is underlined at its place and its message shows on hover.
- **Output.** Printed lines as they happen; an error exactly as the terminal shows it;
  `запитай` shows its question with an input field right in the output, and Enter sends the
  answer; "Готово" / "Done" when the program ends, "Зупинено" / "Stopped" after Stop.
- **Picture.** The SVG from the core, scaled to fit with its proportions kept; it replays its
  animation after every run. Before the first drawing the panel invites you to open an example.
  Two buttons download the picture as SVG or as PNG (the finished picture at twice the canvas
  size).
- **Examples.** flag, sun, star, house, spiral, fizzbuzz, guess and functions — the files from
  `examples/`, built into the page.
- **UA/EN.** Switches the interface text and the language of error messages; remembered.
- **Share.** Puts the program into the link (`#code=…`) and copies the link; opening such a link
  puts that program in the editor without running it.
- **Autosave.** The editor text is kept in the browser; a shared link takes priority over it.
- **Safety.** A run may take at most 10 000 000 steps (loop iterations and calls); the core
  already limits a drawing to 50 000 shapes; Stop ends the worker, and a fresh one serves the
  next run.

## Architecture

```
crates/yu-wasm/   the core as a WebAssembly module: a small hand-written bridge,
                  no dependencies besides yu-core
studio/           Vite + TypeScript page, no framework
  src/main.ts        layout, buttons and state
  src/editor.ts      CodeMirror: Yu highlighting, completion, errors, keys
  src/runner.ts      the page side of the worker: run, stop, answers to запитай
  src/worker.ts      loads the module, runs a program, reports back
  src/share.ts       program to link and back
  src/i18n.ts        interface text in Ukrainian and English
  src/theme.css      dark and light themes as CSS variables
  tests/             unit tests (node --test) and the browser smoke test (Playwright)
```

### The core gains a reference

The editor takes its words from the core, so the two never drift apart:

- `keywords.rs` keeps every spelling in one table, `SPELLINGS`, which `keyword()` reads.
- Every built-in gets a signature and a one-line description in both languages
  (`Builtin::signature(lang)`, `Builtin::doc(lang)`), e.g. `коло(x, y, радіус)` — «зафарбоване
  коло».

### The bridge

`yu-wasm` compiles to `wasm32-unknown-unknown` as a `cdylib`.

| Export | Meaning |
|---|---|
| `alloc(len) -> ptr`, `dealloc(ptr, len)` | memory for strings passed in and out |
| `run(src_ptr, src_len, lang, seed_lo, seed_hi) -> ptr` | runs the program in a new session; returns a pointer to a UTF-8 JSON result prefixed with its length (u32, little-endian) |
| `names() -> ptr` | keywords and built-ins with signatures and descriptions in both languages, as length-prefixed JSON |

| Import (from JS) | Meaning |
|---|---|
| `host_print(ptr, len)` | one printed line |
| `host_ask(prompt_ptr, prompt_len, out_ptr, out_cap) -> i32` | waits for the answer; returns its length in bytes, or -1 when there is none |

The result JSON is `{"ok":true,"drew":true,"svg":"<svg…>"}` or
`{"ok":false,"drew":false,"svg":"<svg…>","error":{"text":"Помилка в рядку 2: …","from":12,"to":16}}`,
where `from` and `to` are UTF-16 offsets, the unit the editor counts in. The logic lives in plain
Rust functions tested natively; the `extern "C"` layer only moves bytes.

A run uses the chosen language, a step budget of 10 000 000 and a seed from the page
(`crypto.getRandomValues`).

The module's stack is large enough that 1000 nested Yu calls end with the friendly "too many
nested calls" error, never a crash, and a test proves it. A trap inside the module is reported
as an internal error.

### Worker protocol

- The page creates the worker and hands it a `SharedArrayBuffer`: two `Int32` cells (state,
  answer length) followed by 4096 bytes for the answer.
- Page to worker: `{type: "run", code, lang, seed}`.
- Worker to page: `{type: "print", line}`, `{type: "ask", prompt}`, `{type: "done", result}`,
  `{type: "crash", message}`.
- `запитай`: the worker posts `ask` and waits with `Atomics.wait`; the page writes the answer,
  sets the state and calls `Atomics.notify`.
- Stop: `worker.terminate()`, then a new worker.
- `SharedArrayBuffer` needs a cross-origin isolated page, so every response carries
  `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp`
  (Vite's dev and preview servers and Vercel). Every asset, fonts included, comes from the site
  itself. Without isolation `запитай` gets no answer and the output says why.

### Look

| Token | Dark | Light |
|---|---|---|
| page | `#0d1117` | `#f6f8fa` |
| panel | `#161b22` | `#ffffff` |
| border | `#30363d` | `#d0d7de` |
| text | `#e6edf3` | `#1f2328` |
| muted text | `#8b949e` | `#57606a` |
| accent (Run, focus) | `#a78bfa` | `#0057b7` |
| keyword | `#a78bfa` | `#8250df` |
| built-in | `#79c0ff` | `#0550ae` |
| text in quotes | `#ffd700` | `#9a6700` |
| number | `#f0883e` | `#953800` |
| comment | `#8b949e` | `#6e7781` |
| error | `#f85149` | `#cf222e` |

Fonts: Inter for the interface and JetBrains Mono for code, both self-hosted; both cover
Cyrillic. Controls have an 8 px radius, panels 12 px. Motion stays small — a 150 ms fade for
menus and the toast; the picture's own animation is the show.

### Share format

`#code=` followed by the base64url of the program compressed with
`CompressionStream("deflate-raw")`; decoding reverses it. A broken link shows a short message
and leaves the editor as it was.

## Deploy

`npm run build` in `studio/` first builds the module
(`cargo build --release --target wasm32-unknown-unknown -p yu-wasm`) and copies it into the site,
then builds the page into `studio/dist`. That folder, with a `vercel.json` that sets the two
headers, is deployed with the Vercel CLI as the project `yu-studio` (`yu-studio.vercel.app` if
the name is free). The README links to the Studio.

## Testing

- Rust: the bridge's JSON (success, an error with UTF-16 offsets, escaping), `names`, the keyword
  table, a signature and a description for every built-in, and deep recursion ending in the
  friendly error.
- Node unit tests (`node --test`; Node 24 runs TypeScript directly): the share format round-trips;
  the Yu highlighter tells keywords, built-ins, text, numbers and comments apart.
- Browser smoke test (Playwright, Chromium): run the star example and find a polygon in the
  picture; run a program with an error and see it in the output and underlined in the editor;
  answer `запитай` and see the greeting; start an endless loop and end it with Stop.
- CI: a `studio` job builds the module and the page and runs the Node tests and the smoke test.

## Not in this milestone

Accounts and programs saved on a server, several files, a step-by-step debugger, the Yu book and
the release (milestone 4), deploying automatically from GitHub.
