mod common;

use std::fs;
use std::path::Path;

use common::Capture;
use yu_core::{Lang, Options};

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
        let (expected_file, actual) = match yu_core::run(&src, &mut host, Options::default()) {
            Ok(()) => (path.with_extension("out"), host.out.join("\n")),
            Err(e) => (path.with_extension("err"), e.render(&src, Lang::Uk)),
        };
        let expected = fs::read_to_string(&expected_file)
            .unwrap_or_else(|_| panic!("missing {}", expected_file.display()));
        assert_eq!(
            actual.trim_end(),
            expected.replace("\r\n", "\n").trim_end(),
            "{}",
            path.display()
        );
        checked += 1;
    }
    assert!(checked >= 6, "only {checked} examples found");
}
