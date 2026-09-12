# Yu — language design, v1

Date: 2026-09-11 · Author: Yurii Dmytrenko · Status: approved for planning

## Summary

Yu is a small, dynamically typed programming language whose home language is Ukrainian.
Every keyword and built-in has a Ukrainian name and an English alias, and the two can be
mixed in one program. Blocks are marked by indentation, as in Python. Drawing is built in,
Python-Turtle style, so a first program can put a circle on the screen in one line.

One Rust core runs in two places: a terminal (`yu` command) and the browser (WebAssembly
playground). The core never draws itself; it emits drawing commands that each host renders.
Source files use the `.yu` extension.

```
# Yu
бал = 0
повтори 4 рази:
    вперед(100)
    праворуч(90)

якщо бал > 10:
    скажи("Молодець!")
інакше:
    скажи("Ще трохи")
```

The same program in English keywords — `repeat 4 times:`, `forward(100)`, `if score > 10:`,
`say("Well done!")` — runs identically, and the two may be mixed.

## Goals (v1)

- Syntax simple enough for a first program: no semicolons, no declarations, indentation blocks.
- Bilingual: Ukrainian first, English aliases for every keyword and built-in.
- Drawing in one line: shapes plus a turtle, colours by Ukrainian or English name.
- Errors a beginner can act on: the line, a caret under the problem, a hint, in Ukrainian
  (default) or English.
- `yu` CLI: run a file, interactive REPL, export the drawing to SVG.
- Web playground: editor with highlighting, canvas, console, examples, share-by-link.
- Tests and CI from the first commit.

## Non-goals (v1)

Static types, modules/imports, classes, dictionaries, string interpolation, files and
networking, animation, a bytecode VM, a package manager, a VS Code extension. Several of these
are planned for later milestones (see Milestones).

## The language

### Source text

- UTF-8. Comments start with `#` and run to the end of the line.
- Identifiers: Unicode letters (Latin, Cyrillic), digits and `_`, not starting with a digit.
  An apostrophe (`'`, `’` or `ʼ`) is allowed between two letters, so `ім'я` and `пам'ять` are
  valid names. Strings therefore use double quotes only.
- Numbers: 64-bit floats (`3`, `2.5`, `1_000`); whole numbers print without `.0`.
- Strings: `"…"` with escapes `\n`, `\t`, `\"`, `\\`.
- Statements end at a newline. Inside `()` and `[]` newlines are ignored.
- Blocks: a line ending in `:` opens a block; the block is the following lines indented deeper.
  The lexer turns indentation changes into INDENT/DEDENT tokens. Mixing tabs and spaces in one
  file is an error with a clear message; a tab counts as 4 spaces.

### Keywords

Reserved in both languages; using one as a name is an error that suggests renaming.

| Ukrainian | English | Meaning |
|---|---|---|
| `якщо` | `if` | condition |
| `інакше`, `інакше якщо` | `else`, `else if` | alternatives |
| `поки` | `while` | loop while true |
| `повтори N рази:` (`раз`, `рази`, `разів`, or just `повтори N:`) | `repeat N times:` / `repeat N:` | loop N times |
| `для x від A до B:` (optional `крок S`) | `for x from A to B:` (`step S`) | inclusive counting loop |
| `для x у список:` (`у` or `в`) | `for x in list:` | loop over a list |
| `функція` | `function` | define a function |
| `поверни` | `return` | return a value |
| `стоп` | `break` | leave a loop |
| `далі` | `continue` | next iteration |
| `і`, `або`, `не` | `and`, `or`, `not` | logic |
| `так`, `ні` | `true`, `false` | booleans |
| `нічого` | `nothing` | no value |

### Values and semantics

- Types: number, text, boolean, nothing, list, function.
- Variables are created by assignment: `бал = 0`. Assigning to a name that already exists in an
  enclosing scope updates it; otherwise the name is created in the current scope. No `global`.
- Lists: `[1, 2, 3]`, indexed from **1** (`список[1]` is the first item), like the counting
  loop that starts at 1. An out-of-range index is an error that names the list length.
- Operators: `+ - * / %`, comparisons `== != < <= > >=`, `і або не`. `+` with text on either
  side joins as text (`"Бал: " + 5` → `Бал: 5`). Division by zero is an error.
- Falsy values: `ні`, `нічого`, `0`, `""`, `[]`. Everything else is true.
- Functions are closures; recursion deeper than 1 000 calls is a friendly error, not a crash.

### Built-ins

| Ukrainian | English | |
|---|---|---|
| `скажи(…)` | `say(…)` | print arguments separated by spaces |
| `запитай("…")` | `ask("…")` | read a line (terminal) or show a prompt (playground) |
| `довжина(x)` | `length(x)` | length of text or list |
| `число(x)`, `текст(x)` | `number(x)`, `text(x)` | conversions |
| `випадкове(a, b)` | `random(a, b)` | whole number between a and b inclusive |
| `округли(x)` | `round(x)` | nearest whole number |
| `додай(список, x)` | `append(list, x)` | add to the end of a list |

Drawing — canvas 600×400, white, origin top-left, y grows downward:

| Ukrainian | English | |
|---|---|---|
| `полотно(ш, в)` | `canvas(w, h)` | resize the canvas |
| `фон("колір")` | `background("colour")` | fill the canvas |
| `колір("колір")` | `color("colour")` | colour for the next shapes and turtle lines |
| `товщина(n)` | `thickness(n)` | line width |
| `коло(x, y, радіус)` | `circle(x, y, r)` | filled circle |
| `прямокутник(x, y, ш, в)` | `rect(x, y, w, h)` | filled rectangle |
| `лінія(x1, y1, x2, y2)` | `line(x1, y1, x2, y2)` | line |
| `напис("…", x, y)` | `label("…", x, y)` | text |
| `вперед(n)`, `назад(n)` | `forward(n)`, `back(n)` | move the turtle |
| `праворуч(°)`, `ліворуч(°)` | `right(°)`, `left(°)` | turn the turtle |
| `підніми_перо()`, `опусти_перо()` | `pen_up()`, `pen_down()` | stop / resume drawing |

The turtle starts in the centre, facing right, pen down. Colours: `червоний`, `помаранчевий`,
`жовтий`, `зелений`, `блакитний`, `синій`, `фіолетовий`, `рожевий`, `білий`, `чорний`,
`сірий`, `коричневий` in any gender form (`червона`, `червоне`), their English names, or
`"#rrggbb"`.

### Errors

Every error carries a source span and renders the same way in both hosts:

```
Помилка в рядку 3: невідома назва «скаж»
  3 | скаж("Привіт")
    | ^^^^
  Можливо, ти мав на увазі «скажи»?
```

Kinds: syntax (unexpected token, bad indentation, unclosed string or bracket), unknown name
(with a "did you mean" drawn from names in scope and built-ins in both languages), wrong type,
wrong number of arguments, index out of range, division by zero, recursion too deep, and — in
the playground — "the program runs too long". Messages come from one catalogue with a
Ukrainian and an English text for each; Ukrainian is the default, `--lang en` or the
playground switch selects English.

## Architecture

A Cargo workspace plus a web app:

```
yu/
  crates/yu-core/   lexer → parser (recursive descent, Pratt for expressions) → AST →
                    tree-walking interpreter; values, built-ins, diagnostics.
                    No I/O: it talks to a `Host` trait (print, ask).
  crates/yu-cli/    the `yu` binary: `yu run file.yu [--svg out.svg] [--lang en]`,
                    `yu` alone opens the REPL; coloured diagnostics.
  crates/yu-wasm/   hand-written wasm bridge: run(source, lang) → output lines, the SVG,
                    error.
  studio/           Vite + TypeScript + CodeMirror 6. Runs yu-wasm in a Web Worker
                    (Stop = terminate the worker; a budget of 10 million evaluation
                    steps catches endless loops).
  examples/         gallery programs in both languages; they double as golden tests.
  docs/             the Yu book (Ukrainian and English) and design documents.
```

Drawing flows one way: the interpreter records `DrawCmd`s in the session's `Drawing`, each
shape carrying its own colour, and one renderer in the core turns them into an animated SVG.
The CLI saves that SVG and the playground shows it, so a picture looks the same everywhere.
The turtle is state inside the core and records `line` commands. Details:
[drawing design](2026-09-11-yu-drawing-design.md).

Toolchain: Rust stable on the Windows GNU host (no Visual Studio needed) with the
`wasm32-unknown-unknown` target; a small hand-written bridge to JavaScript for the browser build.

## Playground

A single page in the palette of the GitHub profile card (background `#0d1117`, panels
`#161b22`, accent `#a78bfa`, Ukrainian blue and yellow as highlights): editor on the left,
canvas and console on the right. Run with the button or Ctrl+Enter, Stop, an examples menu
(house, sun, star, spiral, FizzBuzz in Ukrainian, guess-the-number), a UA/EN switch for the
interface and error messages, and Share, which puts the compressed program in the URL.
Errors are underlined in the editor at their span. Deployed to Vercel. Details: [Yu Studio design](2026-09-12-yu-studio-design.md).

## Testing

- Unit tests for the lexer (indentation, apostrophes, both keyword sets), parser and
  interpreter semantics.
- Golden tests: every `examples/*.yu` has expected console output and, when it draws, an
  expected SVG.
- Snapshot tests of error messages in both languages.
- Playground: a Playwright smoke test runs an example and checks the canvas is not blank and
  an error example shows its message.
- CI (GitHub Actions): `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, the wasm
  build, the playground build and the smoke test.

## Milestones

1. **Core** — lexer, parser, interpreter, built-ins, diagnostics, `yu run` and the REPL, tests.
2. **Drawing** — drawing commands, turtle, colours, SVG export.
3. **Playground** — wasm build, the web app, examples, share links, deploy.
4. **v1.0** — README with a GIF of code turning into a picture, the Yu book, release.

Later: v1.1 animation (`кожен кадр:` / `every frame:`), dictionaries, string interpolation;
v2 bytecode VM with benchmarks against Python, a VS Code extension, prebuilt binaries.

## Risks

- Indentation edge cases (tabs, blank lines, comments) — handled in the lexer with tests and
  explicit errors.
- Short Ukrainian words reserved as keywords (`до`, `у`, `в`, `і`) may clash with names users
  want; the error explains and suggests another name.
- Endless loops in the browser — the Web Worker and the step budget keep the page responsive.
