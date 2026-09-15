# Yu Studio 2 — design (milestone 4)

Date: 2026-09-15 · Author: Yurii Dmytrenko · Status: approved for planning

Extends the [Yu Studio design](2026-09-12-yu-studio-design.md). Where they differ, this document
wins.

## Summary

Studio 2 makes Yu Studio look and work like a small code editor in the colours of the Yu logo.
A menu bar holds every command with its shortcut. Programs open from and save to files on the
computer. A sidebar shows the examples and the library: every word of Yu with its meaning and a
working example. The picture and the text output share one panel with two tabs, Picture and
Terminal, and after a run the Studio shows the one the program used. The README gets the logo and
a GIF of the Studio at work. The Yu book and the release move to milestone 5.

## The logo

Yurii's logo is a square filled with `#279F7C`, with sharp corners and "YU" in white bold
sans-serif letters at the bottom left. Measured on his picture: the letters span 8.6–64.6 % of the
width, they are 31 % of the side tall, and their baseline sits 12.8 % of the side above the bottom
edge.

`studio/public/logo.svg` draws it with paths on a 64 × 64 grid, so it needs no font. It is the
favicon, it opens the menu bar at 22 px, and it heads the README. It replaces the turtle favicon;
the turtle stays in the pictures, where the core draws it.

## What the user sees

### Layout

```
┌──────────────────────────────────────────────────────────────────────────┐
│ [YU] Файл  Правка  Вигляд  Запуск  Довідка         Поділитися  ▶ Запустити │  menu bar
├────┬───────────────┬──────────────────────────┬──────────────────────────┤
│ Ex │ Приклади      │ star.yu ●                │ Малюнок   Термінал •      │
│ Li │   or          │                          │                          │
│    │ Бібліотека    │ the editor               │ the picture or the       │
│    │               │                          │ terminal                 │
├────┴───────────────┴──────────────────────────┴──────────────────────────┤
│ star.yu ●   Рядок 4, стовпчик 12                          Готово   UA   ◐ │  status bar
└──────────────────────────────────────────────────────────────────────────┘
```

- Panels meet edge to edge and are divided by 1 px lines, as in VS Code. The activity bar is
  44 px wide, the sidebar 264 px, the output 40 % of the rest (at least 320 px); the editor takes
  what is left.
- Narrower than 900 px, the activity bar hides and Examples and Library open from the View menu
  as a drawer over the editor. The output goes under the editor: the editor is 55 vh tall, the
  output at least 320 px.
- Narrower than 640 px, Share lives only in the File menu and Run shows just ▶ (its name stays
  for screen readers). The menu titles scroll sideways if they still don't fit.

### Menu bar

The logo, five menus, then Share and Run; Run becomes Stop while a program runs. A menu opens on
click; while one is open, pointing at another title switches to it. The arrow keys move through
the items and between the menus, Enter runs an item, and Escape closes the menu and puts the
focus back where it was. Every item shows its shortcut on the right: Ctrl on Windows and Linux,
⌘ on a Mac.

| Menu | Items |
|---|---|
| Файл / File | Новий файл · Відкрити… Ctrl+O · Зберегти Ctrl+S · Зберегти як… Ctrl+Shift+S · — · Поділитися посиланням · Зберегти малюнок SVG · Зберегти малюнок PNG |
| Правка / Edit | Скасувати Ctrl+Z · Повторити Ctrl+Y · — · Знайти й замінити Ctrl+F · — · Закоментувати рядок Ctrl+/ · Виділити все Ctrl+A |
| Вигляд / View | Приклади · Бібліотека · Бічна панель Ctrl+B · — · Малюнок · Термінал · — · Світла тема (or Темна тема) · English (or Українська) |
| Запуск / Run | Запустити (or Зупинити) Ctrl+Enter |
| Довідка / Help | Бібліотека слів · Yu на GitHub |

- Shortcuts are matched by the physical key, so they also work with the Ukrainian keyboard
  layout. A shortcut the browser keeps for itself still works from the menu.
- The Edit items call the editor's own commands, and their keys are the editor's.
- Find and replace comes from `@codemirror/search`, styled with the theme.

### Files

- **Відкрити** picks a `.yu` file (or any text file). **Зберегти** writes back to the same file;
  the first time, and after **Зберегти як**, it asks where. In Chrome and Edge this uses the File
  System Access API (`showOpenFilePicker`, `showSaveFilePicker`). In other browsers Open uses a
  file input and Save downloads the file under its name.
- The tab above the editor shows the file name. A dot (●) means unsaved changes; the browser tab
  title starts with it too: `● star.yu — Yu Studio`.
- Names: an example opens under its file name (`star.yu`), a shared link as `програма.yu`
  (`program.yu`), a new file as `без назви.yu` (`untitled.yu`).
- New file, Open and opening an example ask first when they would replace unsaved changes, with
  the browser's own confirm dialog.
- Autosave keeps the text, the name and whether it is saved. The link to the file on disk does
  not survive a reload, so the next Save asks where, suggesting the same name.
- A file over 1 MB is refused with a message. A cancelled file dialog does nothing. A failed
  write shows its reason and keeps the dot.
- Pictures are saved under the program's name: `star.yu` gives `star.svg` and `star.png`.

### One output: Picture and Terminal

Two tabs over one panel (`role="tablist"`; the arrow keys move between the tabs). After a run the
Studio opens:

- **Terminal** as soon as the program calls `запитай`, with the answer field focused;
- **Terminal** when the run ends with an error or a crash, or is stopped;
- **Picture** when the program drew something;
- **Terminal** otherwise.

New content in the hidden tab puts a dot on its title until the tab is opened. The tabs can
always be clicked; the rule applies again at the next run.

- **Terminal.** Each run starts clean with the command line `› yu star.yu`, then the printed
  lines as they happen, `запитай` with its answer field, an error exactly as the terminal shows
  it, and a last muted line, "Готово" or "Зупинено".
- **Picture.** As in milestone 3: the dot-grid table with the picture lying on it like a sheet of
  paper. The SVG and PNG buttons sit at the right of the tab strip while Picture is open.

### Sidebar

The activity bar has two buttons, **Приклади** and **Бібліотека**. A click shows that view;
clicking the open one hides the sidebar, and so does Ctrl+B. The choice is remembered; on the
first visit on a wide screen Examples is open.

**Приклади** lists the eleven examples, each with its title and file name, and marks the open
one.

**Бібліотека** has a search field and the words of Yu in sections:

| Section | Words |
|---|---|
| Основи / Basics | скажи, запитай, текст, число, округли, випадкове |
| Умови / Conditions | якщо; інакше (and інакше якщо); і · або · не; так · ні; нічого |
| Цикли / Loops | повтори … разів; поки; для … від … до (with крок); для … у; стоп; далі |
| Функції / Functions | функція, поверни |
| Списки / Lists | довжина, додай |
| Малювання / Drawing | полотно, фон, колір, товщина, коло, прямокутник, лінія, напис |
| Черепашка / Turtle | вперед, назад, праворуч, ліворуч, підніми_перо, опусти_перо, почни_заливку, заверши_заливку |
| Кольори / Colours | the twelve named colours as swatches with both names, and `випадковий` / `random` |

A closed entry shows its word in both languages. An open entry shows how to write it
(`коло(x, y, радіус)`), what it does in one or two sentences, an example of two to seven lines,
and two buttons:

- **Вставити** puts the example into the program. It goes on a new line after the cursor's line,
  at that line's indentation, one level deeper after a line that ends in `:`. When the cursor's
  line is blank, the example takes its place.
- **Спробувати** runs the example inside the entry and shows what it printed and, if it drew,
  the picture. The picture is shown as an image, so its animation styles never mix with the main
  picture. An example that asks gets the sample answer «Юрій» (Yurii).

A click on a colour swatch inserts its name in the interface language, in quotes, at the cursor;
`випадковий` / `random` has a swatch of its own, drawn by the page. The interface language picks
the text and the example (Ukrainian or English keywords). Search matches either spelling, the
"how to write it" line and the description, ignoring case. Sections without matches hide, and
"Нічого не знайшлося" shows when nothing matches.

### Status bar

A band in the logo green with the file name and its dot, the cursor position (`Рядок 4,
стовпчик 12` / `Ln 4, Col 12`), the run state (`Працює…` with a pulse, `Готово`, `Зупинено`,
`Помилка`), and two buttons: UA/EN and the theme.

### Themes

Dark green (the default) and light green, both built on the logo's green. The theme switch is in
the View menu and the status bar, and the choice is remembered.

## Look

| Token | Dark | Light | Used for |
|---|---|---|---|
| `--page` | `#0e1c17` | `#e3f1ea` | menu bar, activity bar, sidebar, tab strips |
| `--surface` | `#11241d` | `#f6fbf8` | editor, terminal, picture table |
| `--raised` | `#173128` | `#ffffff` | menus, library entries, the find panel |
| `--border` | `#214035` | `#bcd9cb` | 1 px lines |
| `--grid` | `#1f3b31` | `#cde3d8` | the dots of the picture table |
| `--text` | `#e3f2eb` | `#0f2a20` | text |
| `--muted` | `#93b8a9` | `#4a6b5e` | secondary text |
| `--accent` | `#279f7c` | `#279f7c` | the logo green as a fill: status bar, Run |
| `--on-accent` | `#04130d` | `#04130d` | text on the logo green |
| `--accent-text` | `#4cc59c` | `#17775a` | green text and marks on panels: focus ring, the unsaved dot, the open tab's line, the terminal's `›` |
| `--error` | `#ff7b72` | `#c62828` | errors |
| `--stop` | `#d1242f` | `#d1242f` | the Stop button, with white text |
| keyword | `#5fd4a8` | `#0f7a5a` | |
| built-in | `#7dd3fc` | `#0550ae` | |
| text in quotes | `#ffd700` | `#8a5a00` | |
| number | `#fdba74` | `#a33a00` | |
| comment | `#86a99a` | `#5b7a6d` | |

- Contrast: text colours reach 4.5 : 1 on every surface they sit on, and marks reach 3 : 1. A
  unit test reads `theme.css` and checks the pairs.
- Fonts stay Inter for the interface and JetBrains Mono for code.
- Menus and the toast fade in over 120 ms. Apart from them only the picture's own animation and
  the running pulse move, and reduced motion is respected as in milestone 3.

## Architecture

```
crates/yu-core/src/library.rs   the library: every word, what it does, an example (uk and en)
crates/yu-wasm/src/lib.rs       + library_json() and the `library` export
studio/public/logo.svg          the logo, also the favicon
studio/src/
  main.ts       start-up and wiring: the open file, runs, autosave
  commands.ts   the command table, keys matched by physical key, key labels
  menu.ts       the menu bar
  files.ts      open and save: the File System Access API, or a file input and a download
  sidebar.ts    the activity bar and its two views
  library.ts    the library view: search, entries, Insert, Try
  output.ts     the Picture and Terminal tabs and the rule that picks one
  editor.ts     + search, inserting an example, the cursor position
  bridge.ts     + library()
  i18n.ts       + the new interface text
  theme.css     the green themes and the editor layout
studio/scripts/gif.mjs          records docs/studio.gif
```

### The core gains the library

`library.rs` holds the library, so the Studio, a future book and the CLI can read the same words.

- `Section`: an id and a title in both languages.
- `Entry`:
  - `words`: the keywords or the built-in it covers;
  - `name`: e.g. `["повтори … разів", "repeat … times"]`; a built-in's names come from `NAMES`;
  - `form`: how to write it; a built-in's comes from `Builtin::signature`;
  - `text`: what it does, one or two sentences per language;
  - `example`: a short program per language.
- The entries in the order of the table above; the colours come from `draw::PALETTE`.

Tests: every built-in in `NAMES` and every `Keyword` has an entry; every example runs without an
error in both languages, with a host that answers «Юрій»; every example contains, in its own
language, a spelling of each keyword of its entry, or the built-in's name.

### The bridge gains `library()`

`yu-wasm` exports `library() -> ptr`, length-prefixed JSON like `names()`:

```json
{"sections":[{"id":"basics","uk":"Основи","en":"Basics","entries":[
  {"uk":"скажи","en":"say","form_uk":"скажи(…)","form_en":"say(…)",
   "text_uk":"…","text_en":"…","ex_uk":"…","ex_en":"…"}]}],
 "colors":[{"uk":"червоний","en":"red","hex":"#ef4444"}]}
```

### The page

- The page also loads the module on its main thread, for the editor's words and the library, and
  runs the library's examples there: they are short, and the core's tests prove they finish.
  Programs from the editor still run in the worker.
- Commands live in one table in `commands.ts`: an id, the words in both languages, the keys and
  what the command does. The menu, the global key handler and the key labels all read it. The
  global handler skips keys the editor has already handled.

## Testing

- **Rust:** the library's completeness and examples (above) and the shape of `library_json`; the
  existing tests stay.
- **Node unit tests:** keys matched by physical key and their labels (Ctrl or ⌘); the rule that
  picks the tab; library search; where Insert puts an example and how it indents it; file names
  (`star.yu` → `star.svg`); theme contrast.
- **Browser smoke test (Playwright):**
  - the milestone 3 checks moved to the new layout: the page is isolated; the star example draws
    and opens Picture; an error opens Terminal and is underlined; `запитай` is answered in
    Terminal; Stop ends an endless loop; English switches the interface and the errors;
  - the File menu shows Save with Ctrl+S, and Escape closes it;
  - Ctrl+S saves through a stubbed `showSaveFilePicker`, and a second Ctrl+S writes to the same
    file without asking;
  - without the API, Ctrl+S downloads `star.yu`, and Open reads a chosen file and shows its name;
  - the library finds «коло», Try shows a circle, Insert adds the example to the program;
  - a program that only prints opens Terminal.
- **CI:** the existing jobs run the new tests.

## README and GIF

- The logo at the top.
- Under the Studio link, `docs/studio.gif`: the star example drawing itself, then FizzBuzz
  printing in the terminal. `npm run gif` in `studio/` records it from the preview: Playwright
  takes the frames and ffmpeg builds the palette and the GIF, about 960 px wide and at most 5 MB.
- The line about what comes next names the Yu book and the v1.0 release.
- The milestone list in the language design: 4 is Studio 2, 5 is v1.0 (the book and the
  release).

## Deploy

As in milestone 3: `npm run build` in `studio/`, then `vercel deploy dist --prod --yes --project
yu-studio`, and the smoke test runs against https://yu-lang.vercel.app.

## Not in this milestone

Several files open at once, keeping the link to a file across reloads, folders, settings and
custom shortcuts, a debugger, the Yu book and the release (milestone 5).
