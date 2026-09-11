mod common;

use common::{output, Capture};
use yu_core::{Lang, Options, Session, Value};

#[test]
fn hello_world_in_both_languages() {
    assert_eq!(output("скажи(\"Привіт, світе!\")"), vec!["Привіт, світе!"]);
    assert_eq!(output("say(\"Hello, world!\")"), vec!["Hello, world!"]);
}

#[test]
fn both_languages_mix_in_one_program() {
    let src = "for i from 1 to 3:\n    якщо i % 2 == 1:\n        say(i)\n    інакше:\n        скажи(\"парне\")";
    assert_eq!(output(src), vec!["1", "парне", "3"]);
}

#[test]
fn fizzbuzz_in_ukrainian() {
    let src = "для i від 1 до 15:\n    якщо i % 15 == 0:\n        скажи(\"ФізБаз\")\n    інакше якщо i % 3 == 0:\n        скажи(\"Фіз\")\n    інакше якщо i % 5 == 0:\n        скажи(\"Баз\")\n    інакше:\n        скажи(i)";
    let out = output(src);
    assert_eq!(out.len(), 15);
    assert_eq!(
        (
            out[2].as_str(),
            out[4].as_str(),
            out[6].as_str(),
            out[14].as_str()
        ),
        ("Фіз", "Баз", "7", "ФізБаз")
    );
}

#[test]
fn a_session_keeps_variables_between_runs() {
    let mut session = Session::new(Options::default());
    let mut host = Capture::default();
    assert_eq!(session.run("бал = 10", &mut host).unwrap(), None);
    assert_eq!(
        session.run("бал * 2", &mut host).unwrap(),
        Some(Value::Number(20.0))
    );
}

#[test]
fn errors_render_in_the_chosen_language() {
    let src = "скаж(1)";
    let mut host = Capture::default();
    let e = yu_core::run(src, &mut host, Options::default()).unwrap_err();
    assert_eq!(
        e.render(src, Lang::Uk),
        "Помилка в рядку 1: невідома назва «скаж»\n  1 | скаж(1)\n    | ^^^^\n  Можливо, ти мав на увазі «скажи»?"
    );
    assert!(e
        .render(src, Lang::En)
        .starts_with("Error on line 1: unknown name 'скаж'"));
}
