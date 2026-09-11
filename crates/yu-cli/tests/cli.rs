use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn yu() -> Command {
    Command::new(env!("CARGO_BIN_EXE_yu"))
}

fn program(name: &str, src: &str) -> PathBuf {
    let dir = std::env::temp_dir().join("yu-cli-tests");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, src).unwrap();
    path
}

fn text(bytes: Vec<u8>) -> String {
    String::from_utf8(bytes).unwrap().replace("\r\n", "\n")
}

#[test]
fn runs_a_file() {
    let p = program("hello.yu", "скажи(\"Привіт\")\nsay(1 + 1)\n");
    let out = yu().arg("run").arg(&p).output().unwrap();
    assert!(out.status.success());
    assert_eq!(text(out.stdout), "Привіт\n2\n");
}

#[test]
fn errors_go_to_stderr_with_exit_code_1() {
    let p = program("bad.yu", "скаж(1)\n");
    let out = yu().arg(&p).arg("--no-color").output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    let err = text(out.stderr);
    assert!(
        err.starts_with("Помилка в рядку 1: невідома назва «скаж»"),
        "{err}"
    );
    let out = yu().args(["--lang", "en"]).arg(&p).output().unwrap();
    assert!(text(out.stderr).starts_with("Error on line 1"));
}

#[test]
fn bad_options_and_missing_files_exit_with_2() {
    assert_eq!(
        yu().arg("no-such-file.yu").output().unwrap().status.code(),
        Some(2)
    );
    assert_eq!(yu().arg("--wat").output().unwrap().status.code(), Some(2));
}

#[test]
fn ask_reads_stdin() {
    let p = program(
        "ask.yu",
        "ім'я = запитай(\"Ім'я?\")\nскажи(\"Привіт, \" + ім'я)\n",
    );
    let mut child = yu()
        .arg(&p)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all("Юрій\n".as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(text(out.stdout).contains("Привіт, Юрій"));
}

#[test]
fn repl_keeps_state_and_runs_blocks() {
    let mut child = yu()
        .arg("--no-color")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let input = "x = 2\nx * 21\nякщо x > 1:\n    скажи(\"так!\")\n\nвийти\n";
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = text(child.wait_with_output().unwrap().stdout);
    assert!(out.contains("42"), "{out}");
    assert!(out.contains("так!"), "{out}");
}

#[test]
fn svg_saves_the_picture() {
    let p = program("circle.yu", "коло(300, 200, 50)\n");
    let out_file = p.with_extension("svg");
    let _ = std::fs::remove_file(&out_file);
    let out = yu().arg(&p).arg("--svg").arg(&out_file).output().unwrap();
    assert!(out.status.success(), "{}", text(out.stderr));
    let svg = std::fs::read_to_string(&out_file).unwrap();
    assert!(
        svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\""),
        "{svg}"
    );
    assert!(
        svg.contains("<circle cx=\"300\" cy=\"200\" r=\"50\""),
        "{svg}"
    );
}

#[test]
fn a_drawing_without_svg_gets_a_hint() {
    let p = program("hint.yu", "лінія(0, 0, 10, 10)\n");
    let out = yu().arg(&p).arg("--no-color").output().unwrap();
    assert!(out.status.success());
    let err = text(out.stderr);
    assert!(err.contains("Щоб зберегти малюнок: yu "), "{err}");
    let svg_arg = format!("--svg {}", p.with_extension("svg").display());
    assert!(err.contains(&svg_arg), "{err}");
    let out = yu().args(["--lang", "en"]).arg(&p).output().unwrap();
    assert!(text(out.stderr).contains("To save the picture: yu "));
}

#[test]
fn programs_that_draw_nothing_get_no_hint() {
    let p = program("quiet.yu", "скажи(1)\n");
    let out = yu().arg(&p).output().unwrap();
    assert_eq!(text(out.stderr), "");
}

#[test]
fn svg_needs_a_file_name_and_a_program() {
    let p = program("plain.yu", "скажи(1)\n");
    assert_eq!(
        yu().arg(&p).arg("--svg").output().unwrap().status.code(),
        Some(2)
    );
    assert_eq!(
        yu().args(["--svg", "x.svg"])
            .output()
            .unwrap()
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn the_repl_says_once_where_pictures_go() {
    let mut child = yu()
        .arg("--no-color")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all("коло(1, 1, 1)\nколо(2, 2, 2)\nвийти\n".as_bytes())
        .unwrap();
    let err = text(child.wait_with_output().unwrap().stderr);
    assert_eq!(err.matches("--svg").count(), 1, "{err}");
}
