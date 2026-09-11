# Yu — drawing (milestone 2)

Date: 2026-09-11 · Author: Yurii Dmytrenko · Status: approved for planning

Extends the [v1 language design](2026-09-11-yu-language-design.md). Where the two differ, this
document wins.

## Summary

Milestone 2 teaches Yu to draw. A program uses shapes and a turtle; the core records every
shape in a `Drawing`, and a single renderer in the core turns that record into an animated
SVG. The `yu` command saves it with `--svg`. Yu Studio, the next milestone, shows the same SVG
in the browser, so a picture looks identical after `yu`, in the README and in the Studio.

```
колір("синій")
прямокутник(0, 0, 600, 200)
колір("жовтий")
прямокутник(0, 200, 600, 200)
```

## Commands

The canvas is 600×400 and white; the origin is the top-left corner and y grows downward. Every
drawing command returns `нічого`.

| Ukrainian | English | Arguments | Effect |
|---|---|---|---|
| `полотно(ш, в)` | `canvas(w, h)` | 1–4000 each, rounded to whole numbers | resizes the canvas; a turtle that has not been used yet moves to the new centre |
| `фон("колір")` | `background("colour")` | a colour | paints the whole canvas at this point of the program, over anything drawn before |
| `колір("колір")` | `color("colour")` | a colour | colour of the next shapes, labels, lines and fills |
| `товщина(n)` | `thickness(n)` | n ≥ 0 | width of lines drawn by `лінія` and the turtle |
| `коло(x, y, радіус)` | `circle(x, y, r)` | r ≥ 0 | filled circle centred at (x, y) |
| `прямокутник(x, y, ш, в)` | `rect(x, y, w, h)` | w, h ≥ 0 | filled rectangle with its top-left corner at (x, y) |
| `лінія(x1, y1, x2, y2)` | `line(x1, y1, x2, y2)` | | a straight line |
| `напис(значення, x, y)` | `label(value, x, y)` | any value, shown the way `скажи` shows it | 20 px text with its top-left corner at (x, y) |
| `вперед(n)`, `назад(n)` | `forward(n)`, `back(n)` | | moves the turtle; draws a line while the pen is down |
| `праворуч(°)`, `ліворуч(°)` | `right(°)`, `left(°)` | degrees | turns the turtle clockwise / anticlockwise as seen on screen |
| `підніми_перо()`, `опусти_перо()` | `pen_up()`, `pen_down()` | | stops / resumes drawing while the turtle moves |
| `почни_заливку()`, `заверши_заливку()` | `begin_fill()`, `end_fill()` | | fills the shape the turtle walked between the two calls |

Start state: colour black, thickness 2, turtle in the centre facing right with the pen down.
Coordinates and sizes must be numbers (`ExpectedNumber` otherwise); colours must be text.
The new names are built-ins, so like `скажи` they can't be used as variable names. This
reserves common English words such as `line` and `left`; the error for using one already asks
for another name.

### Turtle

- The heading is in degrees: 0 faces right, and turning right adds degrees. Because y grows
  downward, `праворуч(90)` followed by `вперед(10)` moves 10 points down the screen.
- `вперед(n)` moves by n·(cos h, sin h); `назад(n)` is `вперед(-n)`.
- The turtle counts as used once any turtle command runs (moving, turning, the pen, fills).
  Only a used turtle appears in the picture.

### Fills

- `почни_заливку()` starts recording at the turtle's position; every move adds a point, whether
  the pen is up or down.
- `заверши_заливку()` stops recording and fills the recorded polygon with the current colour.
  As in Python Turtle, the fill sits under the lines drawn since `почни_заливку()`; in the
  animation it appears at the moment `заверши_заливку()` runs. A path of fewer than three points
  fills nothing.
- `почни_заливку()` during a recording starts a new one. `заверши_заливку()` without a recording
  is an error. A recording still open when the program ends fills nothing.

### Colours

| Ukrainian | English | Hex |
|---|---|---|
| червоний | red | `#ef4444` |
| помаранчевий | orange | `#f97316` |
| жовтий | yellow | `#ffd700`, the yellow of the Ukrainian flag |
| зелений | green | `#22c55e` |
| блакитний | lightblue | `#38bdf8` |
| синій | blue | `#0057b7`, the blue of the Ukrainian flag |
| фіолетовий | purple | `#8b5cf6` |
| рожевий | pink | `#ec4899` |
| білий | white | `#ffffff` |
| чорний | black | `#000000` |
| сірий | gray, grey | `#9ca3af` |
| коричневий | brown | `#8b5a2b` |

- Every Ukrainian colour works in all its forms: hard stems take `-ий`, `-а`, `-е`, `-і`
  (`червоний`, `червона`, `червоне`, `червоні`); `синій` takes `-ій`, `-я`, `-є`, `-і`.
- Letter case and surrounding spaces don't matter.
- `"#rrggbb"` with six hex digits gives any colour.
- `"випадковий"` (any form) or `"random"` picks one of the eight bright colours, червоний to
  рожевий, never the current one. It follows the run's random seed, like `випадкове`.

### Errors

New kinds, each with a Ukrainian and an English message:

| Kind | Ukrainian | English | Hint |
|---|---|---|---|
| `UnknownColor { name, suggestion }` | невідомий колір «x» | unknown colour 'x' | the nearest colour name ("Можливо, ти мав на увазі «червоний»?"), otherwise the list of colours |
| `ColorNeedsQuotes(name)` | «червоний» — це колір, його треба взяти в лапки | 'red' is a colour and needs quotes | Напиши так: "червоний" / Write it like this: "red" |
| `ExpectedText(ty)` | тут потрібен текст, а маємо: число | expected text, got a number | — |
| `NegativeSize(n)` | розмір не може бути від'ємним: -5 | a size can't be negative: -5 | — |
| `BadCanvas` | полотно може мати від 1 до 4000 точок з кожного боку | the canvas can be 1 to 4000 points on each side | — |
| `FillNotStarted` | заливку ще не почато | no fill has been started | Спершу виклич почни_заливку() / Call begin_fill() first |
| `TooManyShapes` | забагато фігур (понад 50 000) | too many shapes (over 50,000) | Можливо, цикл ніколи не закінчується / A loop may never end |

`ColorNeedsQuotes` replaces `UnknownName` when the unknown name is a colour word, so
`колір(червоний)` explains the missing quotes instead of calling the name unknown. Suggestions
for `UnknownColor` come from the Ukrainian masculine forms and the English names.

## Architecture

- `crates/yu-core/src/draw.rs` — `Rgb`, `DrawCmd`, `Turtle`, `Drawing` (canvas size, commands,
  pen and turtle state, fill recording) and colour parsing. No I/O.
- `crates/yu-core/src/svg.rs` — `render(&Drawing) -> String`, the only renderer.
- `builtins.rs` — sixteen new `Builtin` variants with their names and argument counts; `call`
  receives the run's `&mut Drawing` and checks arguments before touching it.
- `interp.rs` — the interpreter owns the run's `Drawing`; `lookup` raises `ColorNeedsQuotes`.
- `lib.rs` — `Session` keeps the drawing and the random state between runs and exposes
  `drawing()`. Keeping the random state also fixes a milestone-1 bug: the REPL restarted the
  random sequence on every input, so `випадкове(1, 6)` repeated the same number.
- The `Host` trait stays as it is (print, ask): drawing is not I/O, and hosts read the finished
  drawing from the session.

Each `DrawCmd` carries its own colour and width, so a renderer needs no state:

```rust
pub enum DrawCmd {
    Background(Rgb),
    Circle { x: f64, y: f64, r: f64, color: Rgb },
    Rect { x: f64, y: f64, w: f64, h: f64, color: Rgb },
    Line { x1: f64, y1: f64, x2: f64, y2: f64, color: Rgb, width: f64 },
    Label { text: String, x: f64, y: f64, color: Rgb },
    Fill { points: Vec<(f64, f64)>, color: Rgb, slot: usize },
}
```

`Fill::slot` is the fill's place in the animation: the number of commands recorded when
`заверши_заливку()` ran. Every other command's place is its index.

## SVG

```svg
<svg xmlns="http://www.w3.org/2000/svg" width="600" height="400" viewBox="0 0 600 400">
<style>(animation rules)</style>
<rect width="600" height="400" fill="#ffffff"/>
<g class="yu" stroke-linecap="round" stroke-linejoin="round" font-family="…" font-size="20">
(one element per command, in order, then the turtle)
</g>
</svg>
```

- Numbers are rounded to two decimals and printed without trailing zeros; `-0` prints as `0`
  and a non-finite number as `0`, so the file is always valid.
- Animation: every element gets `animation-delay` = place × step, where
  step = min(0.15 s, 4 s ÷ number of places) and the places are the commands plus one for the
  turtle, so any picture finishes in about four seconds. Lines draw themselves
  (`pathLength="1"` with a dash animation one step long); other elements fade in. The fill mode
  is `backwards`: before its turn an element is hidden, afterwards it keeps its plain style, so
  viewers without CSS animation show the finished picture. `prefers-reduced-motion: reduce`
  turns the animation off.
- The turtle, when used, is drawn last: a small green turtle (shell `#22c55e`, head and legs
  `#15803d`) at its final position, rotated to its heading.
- Labels escape `&`, `<` and `>`. The baseline is 16 points below the given y, so y is the top of
  the text.
- The output depends only on the drawing, so tests compare it byte for byte.

## The `yu` command

- `yu file.yu --svg out.svg` (also with `run`) writes the SVG after a successful run. If saving
  fails: `yu: не вдалося зберегти out.svg: …`, exit code 2.
- A program that drew something and ran without `--svg` prints a hint to stderr:
  `Щоб зберегти малюнок: yu file.yu --svg file.svg` (English: `To save the picture: …`), where
  the suggested name is the program's path with `.svg`.
- `--svg` without a path, or without a program file, is a usage error (exit code 2).
- The REPL keeps the drawing in its session and, the first time something is drawn, says once
  that pictures are saved by running a file with `--svg`.

## Examples and README

Five gallery programs in `examples/`, with no randomness, each with an expected `.svg`:
`house.yu`, `sun.yu`, `star.yu`, `spiral.yu` (in English keywords) and `flag.yu`. The golden
test runs every example through a `Session` and compares the console output with `.out` and the
drawing with `.svg`; whichever of the two a program produces must have its expectation file.

The README gets a drawing section: the commands table and a gallery where each example's code
sits next to its animated SVG, which GitHub plays in place.

## Testing

- Unit tests: colour parsing (all forms, English, hex, case, `випадковий` avoiding the current
  colour), turtle geometry (a square returns to its start; `праворуч` turns clockwise on
  screen), canvas re-centring, fill order and slot, every new error kind, SVG details (numbers,
  escaping, delays, the turtle only when used), a session keeping its drawing and random state.
- Golden SVGs for the five examples.
- CLI tests: `--svg` writes the file; the hint appears without `--svg`; `--svg` without a path
  exits with 2.
- A visual check before calling it done: the examples rendered in headless Chromium
  mid-animation and at the end, and the screenshots looked at.

## Not in this milestone

Yu Studio (the next milestone), outline circles, ellipses and polygons as commands, moving the
turtle to a point, text size and alignment, gradients, PNG export and frame animation
(`кожен кадр:`).
