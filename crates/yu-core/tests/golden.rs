mod common;

use std::fs;
use std::path::Path;

use common::Capture;
use yu_core::{svg, Lang, Options, Session};

#[test]
fn examples_match_their_expected_output() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let mut checked = 0;
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("yu") {
            continue;
        }
        let src = fs::read_to_string(&path).unwrap();
        let mut host = Capture::default();
        if let Ok(input) = fs::read_to_string(path.with_extension("in")) {
            host.input = input.lines().map(str::to_string).collect();
        }
        let mut session = Session::new(Options::default());
        match session.run(&src, &mut host) {
            Ok(_) => {
                let drawing = session.drawing();
                expect(&path, "out", &host.out.join("\n"), !host.out.is_empty());
                expect(&path, "svg", &svg::render(drawing), !drawing.is_empty());
            }
            Err(e) => expect(&path, "err", &e.render(&src, Lang::Uk), true),
        }
        checked += 1;
    }
    assert!(checked >= 11, "only {checked} examples found");
}

/// Compares `actual` with the file next to `example`; the file must exist when `produced`.
fn expect(example: &Path, ext: &str, actual: &str, produced: bool) {
    let file = example.with_extension(ext);
    match fs::read_to_string(&file) {
        Ok(expected) => assert_eq!(
            actual.trim_end(),
            expected.replace("\r\n", "\n").trim_end(),
            "{}",
            file.display()
        ),
        Err(_) => assert!(!produced, "missing {}", file.display()),
    }
}
