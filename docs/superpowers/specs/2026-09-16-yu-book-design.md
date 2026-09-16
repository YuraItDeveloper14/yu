# The Yu book — design (milestone 5a)

Date: 2026-09-16 · Author: Yurii Dmytrenko · Status: approved for planning

Extends the [language design](2026-09-11-yu-language-design.md) and the
[Studio 2 design](2026-09-15-yu-studio-2-design.md). Where they differ, this document wins.

## Summary

A beginner's book for Yu in Ukrainian and English: ten chapters and a reference of every word.
The chapters are Markdown in `docs/book/<lang>/`, so they read on GitHub as they are, and they
build into static pages at `yu-lang.vercel.app/book/<lang>/<chapter>/` in the Studio's colours.
Every example has an "Open in Studio" button that puts it in the editor, and a Rust test runs
every example in both languages and compares what it prints with what the book shows.

## Who it is for

Someone who has never written a program: a teenager who opens the Studio and wants a picture or a
game today. One idea per chapter, short paragraphs, an example after every idea, and two
exercises at the end whose answers hide under «Показати відповідь» / "Show the answer".

## The chapters

| File | Ukrainian | English | What it teaches |
|---|---|---|---|
| `01-start` | Перша програма | Your first program | the Studio, `скажи`, running, the terminal, comments |
| `02-values` | Числа й текст | Numbers and text | variables, arithmetic, joining text, `запитай`, `число`, `текст`, `округли`, `випадкове` |
| `03-conditions` | Умови | Conditions | `якщо`, `інакше якщо`, `інакше`, comparisons, `і` `або` `не`, `так` `ні`, `нічого` |
| `04-loops` | Цикли | Loops | `повтори … разів`, `поки`, `для … від … до` with `крок`, `стоп`, `далі` |
| `05-lists` | Списки | Lists | `[…]`, numbering from 1, `довжина`, `додай`, `для … у` |
| `06-functions` | Функції | Functions | `функція`, parameters, `поверни`, calling, one recursive example |
| `07-drawing` | Малювання | Drawing | `полотно`, `фон`, `колір`, `товщина`, `коло`, `прямокутник`, `лінія`, `напис`, saving SVG and PNG |
| `08-turtle` | Черепашка | The turtle | `вперед`, `назад`, `праворуч`, `ліворуч`, the pen, fills, a star and a spiral |
| `09-errors` | Коли щось не так | When something goes wrong | how Yu reports an error, the underline in the Studio, four common mistakes |
| `10-projects` | Три проєкти | Three projects | guess the number, a house, a quiz — each built from earlier chapters |
| `11-words` | Усі слова | All words | generated from the core's library: every word with its form, meaning and example, and the colours |

Both languages hold the same eleven files with the same ids. Ukrainian chapters use Ukrainian
keywords in their examples, English ones use English keywords.

## What a page looks like

- **Header:** the Yu logo (a link to the Studio), the book's name, a search field, the UA/EN
  switch and the theme switch. The theme is the Studio's `yu-theme` in `localStorage`, so the
  choice carries between the Studio and the book.
- **Left:** the list of chapters; the open one is marked. Narrower than 900 px it folds into a
  `<details>` menu above the text.
- **The chapter:** headings, paragraphs at most 70 characters wide, code coloured by the Studio's
  own tokenizer. Under every example: «Відкрити в Студії» / "Open in Studio". Exercises sit in
  `<details>` with the answer inside, and the answer's code is an example like any other.
- **Bottom:** links to the previous and the next chapter.
- **Search** filters chapter titles and headings from a generated index; Enter opens the first
  match. It needs no server.
- **Accessibility:** one `<h1>` per page, a skip link, `header`/`nav`/`main`/`footer` landmarks,
  visible focus rings, and the same colour tokens, so contrast is the Studio's.

## Where things live

```
docs/book/README.md              what the book is, links to both languages
docs/book/uk/01-start.md …       ten written chapters in Ukrainian
docs/book/en/01-start.md …       the same ten in English
docs/book/<lang>/11-words.md     generated from the core's library (never edited by hand)
studio/scripts/build-book.mjs    Markdown to pages, chapter list, search index, sitemap
studio/src/book.ts               the page's script: theme, search, Open in Studio
studio/src/book.css              the book's layout on the Studio's tokens
studio/book/**                   the generated pages (git ignores them)
studio/public/book/search-uk.json, search-en.json   the search index
crates/yu-core/tests/book.rs     runs every example and checks what it prints
```

## The build

- `npm run book` runs the generator; `npm run build` and `npm run dev` run it first, before
  `tsc --noEmit` and Vite.
- The generator loads `public/yu_wasm.wasm` for `names()` and `library()`, reads the Markdown,
  converts it with `markdown-it` 15.0.2 (a build-time dependency, with HTML enabled so
  `<details>` works), colours `yu` code blocks with
  the Studio's `yuParser`, and builds every "Open in Studio" link with `encode()` from
  `src/share.ts`, the same `#code=` format the Studio already reads.
- It writes `studio/book/<lang>/<id>/index.html`, a language picker at `studio/book/index.html`
  (it follows the browser's language and always shows both links), the two search indexes,
  `studio/public/sitemap.xml` and `studio/public/robots.txt`.
- `vite.config.ts` takes every generated HTML file as a build input, so the pages share the
  Studio's hashed CSS and fonts.
- The reference chapter is generated from the core's library into both the page and
  `docs/book/<lang>/11-words.md`. CI runs the build and then `git diff --exit-code docs/book`, so a
  word added to the library without rebuilding the book fails the build.
- The Studio's Help menu gains «Книга Yu» / "The Yu book", which opens the book in the interface
  language.

## The examples are tested

`crates/yu-core/tests/book.rs` walks `docs/book/uk` and `docs/book/en` and, for every fenced
block marked `yu`:

- runs it in a fresh session (step budget 200 000) with a host that answers `запитай` with
  «Юрій» in Ukrainian and "Yurii" in English, and fails with the rendered error if it does not
  finish;
- when a fenced block marked `text` comes straight after the example, the printed lines must
  equal that block exactly, so the book cannot show output the language does not produce;
- checks the example speaks its folder's language: every keyword token is spelled in that
  language and every built-in call uses that language's name;
- checks that every example in the drawing and turtle chapters draws something.

It also checks the structure: both languages have the same eleven chapter ids, every chapter has
exactly one `# ` heading, and every written chapter ends with «Спробуй сам» / "Try it yourself"
holding exactly two exercises, each with its answer inside a `<details>` block, which is what
the test counts.

## Testing

- **Rust:** `book.rs` as above; the existing suites stay green.
- **Node unit tests** (`studio/tests/book.test.ts`): the generator's pure helpers — heading
  slugs, the chapter list with previous and next, the search index's shape, and the HTML escape
  of code.
- **Browser (Playwright):** `/book/` offers both languages; a chapter page shows its heading and
  the chapter list; "Open in Studio" opens the Studio with that example in the editor; the theme
  chosen in the Studio is the one the book opens with; search finds a chapter; no console errors.
- **CI:** the `studio` job builds the book with everything else and fails on a stale
  `docs/book/<lang>/11-words.md`.

## Not in this milestone

The CLI and installing Yu on a computer (that appendix comes with the release, milestone 5b),
PDF or EPUB, languages beyond Ukrainian and English, editing examples on the book page itself.
