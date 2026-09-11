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
