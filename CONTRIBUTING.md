# Як долучитися · Contributing

## Українською

Дякую, що хочеш допомогти Yu. Згодиться все: повідомлення про помилку, ідея, виправлення в книзі
чи код.

### Що встановити

- **Rust** через [rustup](https://rustup.rs). Версію й компоненти rustup візьме з `rust-toolchain.toml`.
- **Node.js 24** для Yu Studio й книги (тека `studio/`).

### Перевірки, які запускає CI

У корені репозиторію:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all
cargo clippy -p yu-wasm --target wasm32-unknown-unknown -- -D warnings
```

У `studio/`:

```bash
npm ci
npm run build                     # WebAssembly, сторінки книги, типи, збірка
npm test                          # модульні тести
npx playwright install chromium
npx playwright test               # Студія, книга, заголовки безпеки, доступність
```

`cargo test --all` запускає кожну програму з `examples/` і кожен приклад книги та звіряє їхній вивід.

### Нове слово мови

1. Слово реалізується в `crates/yu-core`.
2. Слово отримує запис у бібліотеці `crates/yu-core/src/library.rs`: що робить і приклад. Тест не
   пропустить слово без запису, а приклад запускається обома мовами.
3. `npm run build` у `studio/` перезбирає розділ книги «Усі слова» (`docs/book/*/11-words.md`).
   Закоміть ці файли: CI перевіряє, що вони свіжі.

### Коміти й pull request

- Коміт — одне речення англійською про те, що змінилося, наприклад
  `The Studio's Help menu opens the book`.
- Pull request каже, що змінено й навіщо та які перевірки запущено. CI має бути зеленим.

Пишучи в issue чи pull request, дотримуйся [правил спільноти](CODE_OF_CONDUCT.md). Про вразливість
пиши приватно, як сказано в [SECURITY.md](SECURITY.md).

## In English

Thank you for helping Yu. Everything helps: a bug report, an idea, a fix to the book, or code.

### What to install

- **Rust** through [rustup](https://rustup.rs). rustup reads the version and components from
  `rust-toolchain.toml`.
- **Node.js 24** for Yu Studio and the book (the `studio/` folder).

### The checks CI runs

In the repository root:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all
cargo clippy -p yu-wasm --target wasm32-unknown-unknown -- -D warnings
```

In `studio/`:

```bash
npm ci
npm run build                     # WebAssembly, the book's pages, types, the build
npm test                          # unit tests
npx playwright install chromium
npx playwright test               # the Studio, the book, security headers, accessibility
```

`cargo test --all` runs every program in `examples/` and every example of the book, and compares
their output.

### A new word

1. The word is implemented in `crates/yu-core`.
2. The word gets an entry in the library, `crates/yu-core/src/library.rs`: what it does and an
   example. A test fails for a word without an entry, and the example runs in both languages.
3. `npm run build` in `studio/` rebuilds the book's "All words" chapter
   (`docs/book/*/11-words.md`). Commit those files: CI checks they are fresh.

### Commits and pull requests

- A commit is one English sentence about what changed, for example
  `The Studio's Help menu opens the book`.
- A pull request says what changed, why, and which checks ran. CI has to be green.

In issues and pull requests, follow the [code of conduct](CODE_OF_CONDUCT.md). Report a
vulnerability privately, as [SECURITY.md](SECURITY.md) says.
