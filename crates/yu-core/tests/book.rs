//! Every example in the book runs, and prints exactly what the book says it prints.

use std::fs;
use std::path::{Path, PathBuf};

use yu_core::builtins::Builtin;
use yu_core::lexer::lex;
use yu_core::token::TokenKind;
use yu_core::{Host, Lang, Options, Session};

/// One fenced block of a chapter.
struct Block {
    info: String,
    body: String,
    line: usize,
}

/// The fenced blocks of `source`, in order, with the line each one opens on.
fn blocks(source: &str) -> Vec<Block> {
    let mut found = Vec::new();
    let mut lines = source.lines().enumerate();
    while let Some((number, line)) = lines.next() {
        let Some(info) = line.strip_prefix("```") else {
            continue;
        };
        let mut body = String::new();
        for (_, inner) in lines.by_ref() {
            if inner.trim_end() == "```" {
                break;
            }
            body.push_str(inner);
            body.push('\n');
        }
        found.push(Block {
            info: info.trim().to_string(),
            body,
            line: number + 1,
        });
    }
    found
}

/// Answers `запитай` from the book's input block, then with a name.
struct Reader {
    answers: std::vec::IntoIter<String>,
    fallback: &'static str,
    out: Vec<String>,
}

impl Host for Reader {
    fn print(&mut self, line: &str) {
        self.out.push(line.to_string());
    }

    fn ask(&mut self, _prompt: &str) -> Option<String> {
        Some(
            self.answers
                .next()
                .unwrap_or_else(|| self.fallback.to_string()),
        )
    }
}

/// The chapter files of one language, in order.
fn chapters(lang: Lang) -> Vec<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/book")
        .join(lang.pick("uk", "en"));
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|entry| entry.expect("a chapter").path())
        .filter(|path| path.extension().is_some_and(|kind| kind == "md"))
        .collect();
    files.sort();
    files
}

fn stem(path: &Path) -> String {
    path.file_stem()
        .expect("a chapter")
        .to_string_lossy()
        .to_string()
}

/// The lines of `source` outside fenced blocks; a `#` comment in an example is not a heading.
fn prose(source: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in source.lines() {
        if line.starts_with("```") {
            inside = !inside;
            continue;
        }
        if !inside {
            out.push(line);
        }
    }
    out
}

/// Every keyword and built-in in `program` is spelled in `lang`; a program that
/// does not even lex (the chapter about mistakes has those) is left alone.
fn speaks(lang: Lang, program: &str, where_: &str) {
    let Ok(tokens) = lex(program) else {
        return;
    };
    for token in &tokens {
        match &token.kind {
            TokenKind::Kw(_) => {
                let spelled = &program[token.span.start..token.span.end];
                assert_eq!(
                    spelled.is_ascii(),
                    lang == Lang::En,
                    "{where_} uses «{spelled}»"
                );
            }
            TokenKind::Name(name) => {
                if let Some(b) = Builtin::lookup(name) {
                    assert_eq!(b.name(lang), name.as_str(), "{where_}");
                }
            }
            _ => {}
        }
    }
}

#[test]
fn every_example_in_the_book_runs() {
    for lang in [Lang::Uk, Lang::En] {
        for path in chapters(lang) {
            let source = fs::read_to_string(&path).expect("a chapter");
            let found = blocks(&source);
            let draws = matches!(stem(&path).as_str(), "07-drawing" | "08-turtle");
            for (index, block) in found.iter().enumerate() {
                if block.info != "yu" && block.info != "yu-error" {
                    continue;
                }
                let where_ = format!("{}:{}", path.display(), block.line);
                let answers: Vec<String> = found[..index]
                    .last()
                    .filter(|earlier| earlier.info == "input")
                    .map(|earlier| earlier.body.lines().map(str::to_string).collect())
                    .unwrap_or_default();
                let expected = found.get(index + 1).filter(|next| next.info == "text");
                let mut host = Reader {
                    answers: answers.into_iter(),
                    fallback: lang.pick("Юрій", "Yurii"),
                    out: Vec::new(),
                };
                let mut session = Session::new(Options {
                    lang,
                    step_budget: Some(200_000),
                    ..Options::default()
                });
                let result = session.run(&block.body, &mut host);
                if block.info == "yu" {
                    match result {
                        Err(e) => panic!("{where_}\n{}", e.render(&block.body, lang)),
                        Ok(_) => {
                            if let Some(text) = expected {
                                assert_eq!(host.out.join("\n"), text.body.trim_end(), "{where_}");
                            }
                            if draws {
                                assert!(!session.drawing().is_empty(), "{where_} draws nothing");
                            }
                        }
                    }
                } else {
                    let text = expected
                        .unwrap_or_else(|| panic!("{where_}: an error example needs its message"));
                    match result {
                        Ok(_) => panic!("{where_}: this example should fail, but it ran"),
                        Err(e) => assert_eq!(
                            e.render(&block.body, lang).trim_end(),
                            text.body.trim_end(),
                            "{where_}"
                        ),
                    }
                }
                speaks(lang, &block.body, &where_);
            }
        }
    }
}

#[test]
fn the_chapters_are_shaped_alike() {
    let uk: Vec<String> = chapters(Lang::Uk).iter().map(|p| stem(p)).collect();
    let en: Vec<String> = chapters(Lang::En).iter().map(|p| stem(p)).collect();
    assert_eq!(uk, en, "the two languages hold different chapters");
    for lang in [Lang::Uk, Lang::En] {
        for path in chapters(lang) {
            let source = fs::read_to_string(&path).expect("a chapter");
            let titles = prose(&source)
                .iter()
                .filter(|line| line.starts_with("# "))
                .count();
            assert_eq!(titles, 1, "{}", path.display());
            if stem(&path) != "11-words" {
                let exercises = source.matches("<details>").count();
                assert_eq!(exercises, 2, "{} needs two exercises", path.display());
            }
            let found = blocks(&source);
            for (index, block) in found.iter().enumerate() {
                if block.info != "text" {
                    continue;
                }
                let earlier = found[..index]
                    .last()
                    .map(|b| b.info.clone())
                    .unwrap_or_default();
                assert!(
                    earlier == "yu" || earlier == "yu-error",
                    "{}:{} output without an example",
                    path.display(),
                    block.line
                );
            }
        }
    }
}
