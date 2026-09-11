// Shared by several test files; not every file uses every helper.
#![allow(dead_code)]

use std::collections::VecDeque;

use yu_core::{Host, Lang, Options};

#[derive(Default)]
pub struct Capture {
    pub out: Vec<String>,
    pub input: VecDeque<String>,
}

impl Host for Capture {
    fn print(&mut self, line: &str) {
        self.out.push(line.to_string());
    }
    fn ask(&mut self, _prompt: &str) -> Option<String> {
        self.input.pop_front()
    }
}

/// Everything the program printed; panics with the rendered error if it fails.
pub fn output(src: &str) -> Vec<String> {
    let mut host = Capture::default();
    if let Err(e) = yu_core::run(src, &mut host, Options::default()) {
        panic!("{}", e.render(src, Lang::Uk));
    }
    host.out
}
