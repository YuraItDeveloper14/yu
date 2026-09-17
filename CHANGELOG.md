# Зміни · Changelog

Тут записано, що змінюється в Yu. Формат — [Keep a Changelog](https://keepachangelog.com/uk/1.1.0/),
номери версій — за [семантичним версіонуванням](https://semver.org/lang/uk/).

What changes in Yu, version by version. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and versions follow [Semantic Versioning](https://semver.org/).

## [1.0.0] — 2026-09-17

### Українською

Перша стабільна версія.

#### Мова

- Ключові слова й 24 вбудовані команди мають українську та англійську назву, і їх можна змішувати.
- Блоки відступами, як у Python: `якщо`, `поки`, `повтори N разів`, `для … від … до`, `для … у`,
  функції з `поверни`.
- Числа, текст, списки з нумерацією від 1, `так`, `ні` і `нічого`.
- Помилки показують рядок, стрілочки під місцем і підказку «Можливо, ти мав на увазі…» — українською
  або англійською.

#### Малювання

- Полотно, фон, колір, товщина, коло, прямокутник, лінія й напис.
- Черепашка: вперед, назад, повороти, перо й заливка.
- 12 названих кольорів у будь-якому роді, англійською, кодом `#rrggbb` або `випадковий`.
- Анімований SVG: `yu програма.yu --svg малюнок.svg`.

#### Команда `yu`

- Запускає файл або відкриває інтерактивний режим; `--lang en`, `--no-color`, `--version`.
- Готові файли для Windows, Linux і macOS із сумами SHA-256.

#### Yu Studio — https://yu-lang.vercel.app

- Ядро мови як WebAssembly: програма виконується в браузері, встановлювати нічого не треба.
- Редактор із підсвіткою, автодоповненням і відступами; меню, як у VS Code.
- Вивід із вкладками «Малюнок» і «Термінал»; `запитай` чекає відповіді в терміналі.
- Бібліотека 37 слів із поясненнями й прикладами; файли `.yu` відкриваються й зберігаються;
  посилання «Поділитися» несе саму програму.
- Темна й світла зелені теми, українська й англійська.
- Заголовки безпеки й перевірка доступності за WCAG 2.1 AA.

#### Книга Yu — https://yu-lang.vercel.app/book/

- 10 розділів українською й англійською, від першої програми до трьох проєктів, і розділ
  «Усі слова» з ядра мови.
- Кожен приклад відкривається в Студії, а під прикладами з малюванням видно малюнок.
- Тест запускає кожен приклад книги й звіряє його вивід.

### In English

The first stable version.

#### Language

- Keywords and 24 built-in commands have a Ukrainian and an English name, and the two mix freely.
- Blocks by indentation, as in Python: `if`, `while`, `repeat N times`, `for … from … to`, `for … in`,
  and functions with `return`.
- Numbers, text, lists counted from 1, `true`, `false` and `nothing`.
- Errors show the line, arrows under the place and a "Did you mean…?" hint, in Ukrainian or English.

#### Drawing

- Canvas, background, colour, thickness, circle, rectangle, line and label.
- The turtle: forward, back, turns, the pen and fills.
- 12 named colours in any gender form, in English, as `#rrggbb` or `random`.
- Animated SVG: `yu program.yu --svg picture.svg`.

#### The `yu` command

- Runs a file or opens the interactive mode; `--lang en`, `--no-color`, `--version`.
- Ready files for Windows, Linux and macOS with SHA-256 checksums.

#### Yu Studio — https://yu-lang.vercel.app

- The language core as WebAssembly: programs run in the browser with nothing to install.
- An editor with highlighting, completion and indentation, and menus like VS Code's.
- One output with Picture and Terminal tabs; `ask` waits for an answer in Terminal.
- A library of 37 words with explanations and examples; `.yu` files open and save; a Share link
  carries the program itself.
- Dark and light green themes, in Ukrainian and English.
- Security headers, and an accessibility check against WCAG 2.1 AA.

#### The Yu book — https://yu-lang.vercel.app/book/

- 10 chapters in Ukrainian and English, from the first program to three projects, and an
  "All words" chapter built from the core.
- Every example opens in the Studio, and examples that draw show their picture.
- A test runs every example of the book and compares its output.

[1.0.0]: https://github.com/YuraItDeveloper14/yu/releases/tag/v1.0.0
