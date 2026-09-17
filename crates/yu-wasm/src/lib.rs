//! The Yu core as a WebAssembly module for Yu Studio.
//!
//! The logic is plain Rust, tested natively; the `extern "C"` layer at the bottom only moves
//! bytes between JavaScript and these functions.

use yu_core::builtins::NAMES;
use yu_core::draw::PALETTE;
use yu_core::keywords::SPELLINGS;
use yu_core::library::{Section, ENTRIES};
use yu_core::{svg, Host, Lang, Options, Session};

/// Loop iterations and calls a program in the Studio may make before it is stopped.
pub const STEP_BUDGET: u64 = 10_000_000;

/// Text as a JSON string literal.
pub fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The UTF-16 position of a byte offset — the unit the editor counts in.
pub fn utf16_offset(src: &str, byte: usize) -> usize {
    src.char_indices()
        .take_while(|(i, _)| *i < byte)
        .map(|(_, c)| c.len_utf16())
        .sum()
}

/// Runs `src` in a new session and describes the result as JSON for the page.
pub fn run_json(src: &str, lang: Lang, seed: u64, budget: u64, host: &mut dyn Host) -> String {
    let opts = Options {
        lang,
        step_budget: Some(budget),
        seed,
        ..Options::default()
    };
    let mut session = Session::new(opts);
    let result = session.run(src, host);
    let drawing = session.drawing();
    let head = format!(
        "\"drew\":{},\"svg\":{}",
        !drawing.is_empty(),
        json_str(&svg::render(drawing))
    );
    match result {
        Ok(_) => format!("{{\"ok\":true,{head}}}"),
        Err(e) => {
            let from = utf16_offset(src, e.span.start);
            let to = utf16_offset(src, e.span.end.max(e.span.start));
            format!(
                "{{\"ok\":false,{head},\"error\":{{\"text\":{},\"from\":{from},\"to\":{to}}}}}",
                json_str(&e.render(src, lang))
            )
        }
    }
}

/// Keywords and built-ins with their help, as JSON for the editor.
pub fn names_json() -> String {
    let keywords: Vec<String> = SPELLINGS.iter().map(|(w, _)| json_str(w)).collect();
    let builtins: Vec<String> = NAMES
        .iter()
        .map(|(b, uk, en)| {
            format!(
                "{{\"uk\":{},\"en\":{},\"sig_uk\":{},\"sig_en\":{},\"doc_uk\":{},\"doc_en\":{}}}",
                json_str(uk),
                json_str(en),
                json_str(b.signature(Lang::Uk)),
                json_str(b.signature(Lang::En)),
                json_str(b.doc(Lang::Uk)),
                json_str(b.doc(Lang::En))
            )
        })
        .collect();
    format!(
        "{{\"keywords\":[{}],\"builtins\":[{}],\"version\":{}}}",
        keywords.join(","),
        builtins.join(","),
        json_str(env!("CARGO_PKG_VERSION"))
    )
}

/// The library of every word, with the named colours, as JSON for the Studio's sidebar.
pub fn library_json() -> String {
    let sections: Vec<String> = Section::ALL
        .iter()
        .map(|&section| {
            let entries: Vec<String> = ENTRIES
                .iter()
                .filter(|e| e.section == section)
                .map(|e| {
                    format!(
                        "{{\"uk\":{},\"en\":{},\"form_uk\":{},\"form_en\":{},\"text_uk\":{},\"text_en\":{},\"ex_uk\":{},\"ex_en\":{}}}",
                        json_str(e.name(Lang::Uk)),
                        json_str(e.name(Lang::En)),
                        json_str(e.form(Lang::Uk)),
                        json_str(e.form(Lang::En)),
                        json_str(e.text(Lang::Uk)),
                        json_str(e.text(Lang::En)),
                        json_str(e.example(Lang::Uk)),
                        json_str(e.example(Lang::En))
                    )
                })
                .collect();
            format!(
                "{{\"id\":{},\"uk\":{},\"en\":{},\"entries\":[{}]}}",
                json_str(section.id()),
                json_str(section.title(Lang::Uk)),
                json_str(section.title(Lang::En)),
                entries.join(",")
            )
        })
        .collect();
    let colors: Vec<String> = PALETTE
        .iter()
        .map(|c| {
            format!(
                "{{\"uk\":{},\"en\":{},\"hex\":{}}}",
                json_str(&c.name(Lang::Uk)),
                json_str(&c.name(Lang::En)),
                json_str(&c.rgb.hex())
            )
        })
        .collect();
    format!(
        "{{\"sections\":[{}],\"colors\":[{}]}}",
        sections.join(","),
        colors.join(",")
    )
}

#[cfg(target_arch = "wasm32")]
mod ffi {
    use yu_core::{Host, Lang};

    /// Bytes an answer to `запитай` may take; the page's shared buffer has the same room.
    const ANSWER_CAP: usize = 4096;

    // The page passes these in as `env.host_print` and `env.host_ask`.
    #[link(wasm_import_module = "env")]
    extern "C" {
        fn host_print(ptr: *const u8, len: usize);
        fn host_ask(
            prompt_ptr: *const u8,
            prompt_len: usize,
            out_ptr: *mut u8,
            out_cap: usize,
        ) -> i32;
    }

    struct JsHost;

    impl Host for JsHost {
        fn print(&mut self, line: &str) {
            // SAFETY: the page copies `len` bytes at `ptr` before the call returns.
            unsafe { host_print(line.as_ptr(), line.len()) }
        }

        fn ask(&mut self, prompt: &str) -> Option<String> {
            let mut answer = vec![0u8; ANSWER_CAP];
            // SAFETY: the page writes at most `out_cap` bytes into `answer`.
            let len = unsafe {
                host_ask(
                    prompt.as_ptr(),
                    prompt.len(),
                    answer.as_mut_ptr(),
                    answer.len(),
                )
            };
            let len = usize::try_from(len).ok()?;
            answer.truncate(len);
            Some(String::from_utf8_lossy(&answer).into_owned())
        }
    }

    /// Gives `text` to the page as a u32 length and the bytes; the page frees it with
    /// `dealloc(ptr, len + 4)`.
    fn hand_over(text: &str) -> *mut u8 {
        let mut bytes = Vec::with_capacity(text.len() + 4);
        bytes.extend_from_slice(&(text.len() as u32).to_le_bytes());
        bytes.extend_from_slice(text.as_bytes());
        Box::leak(bytes.into_boxed_slice()).as_mut_ptr()
    }

    #[no_mangle]
    pub extern "C" fn alloc(len: usize) -> *mut u8 {
        Box::leak(vec![0u8; len].into_boxed_slice()).as_mut_ptr()
    }

    /// # Safety
    /// `ptr` and `len` must describe one buffer from `alloc`, `run`, `names` or `library`, freed
    /// once.
    #[no_mangle]
    pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
        drop(Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len)));
    }

    /// # Safety
    /// `src_ptr` must point at `src_len` bytes the page wrote into a buffer from `alloc`.
    #[no_mangle]
    pub unsafe extern "C" fn run(
        src_ptr: *const u8,
        src_len: usize,
        lang: u32,
        seed_lo: u32,
        seed_hi: u32,
    ) -> *mut u8 {
        let src = String::from_utf8_lossy(std::slice::from_raw_parts(src_ptr, src_len));
        let lang = if lang == 1 { Lang::En } else { Lang::Uk };
        let seed = (u64::from(seed_hi) << 32) | u64::from(seed_lo);
        hand_over(&super::run_json(
            &src,
            lang,
            seed,
            super::STEP_BUDGET,
            &mut JsHost,
        ))
    }

    #[no_mangle]
    pub extern "C" fn names() -> *mut u8 {
        hand_over(&super::names_json())
    }

    #[no_mangle]
    pub extern "C" fn library() -> *mut u8 {
        hand_over(&super::library_json())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct Capture {
        out: Vec<String>,
    }

    impl Host for Capture {
        fn print(&mut self, line: &str) {
            self.out.push(line.to_string());
        }
        fn ask(&mut self, _prompt: &str) -> Option<String> {
            None
        }
    }

    #[test]
    fn json_strings_are_escaped() {
        assert_eq!(json_str("a\"b\\c\nd\u{1}є"), "\"a\\\"b\\\\c\\nd\\u0001є\"");
    }

    #[test]
    fn offsets_are_counted_in_utf16() {
        assert_eq!(utf16_offset("ab", 1), 1);
        assert_eq!(utf16_offset("Юрій", 4), 2);
        assert_eq!(utf16_offset("😀a", 4), 2);
        assert_eq!(utf16_offset("ab", 99), 2);
    }

    #[test]
    fn a_good_run_reports_output_and_picture() {
        let mut host = Capture::default();
        let json = run_json(
            "скажи(1)\nколо(1, 2, 3)",
            Lang::Uk,
            1,
            STEP_BUDGET,
            &mut host,
        );
        assert!(
            json.starts_with("{\"ok\":true,\"drew\":true,\"svg\":\"<svg xmlns="),
            "{json}"
        );
        assert_eq!(host.out, ["1"]);
    }

    #[test]
    fn a_failed_run_points_at_the_error() {
        let json = run_json(
            "бал = 1\nскаж(бал)",
            Lang::Uk,
            1,
            STEP_BUDGET,
            &mut Capture::default(),
        );
        assert!(json.starts_with("{\"ok\":false,\"drew\":false,"), "{json}");
        assert!(
            json.contains("\"text\":\"Помилка в рядку 2: невідома назва «скаж»\\n"),
            "{json}"
        );
        assert!(json.ends_with("\"from\":8,\"to\":12}}"), "{json}");
    }

    #[test]
    fn endless_loops_stop_at_the_budget() {
        let json = run_json(
            "поки так:\n    x = 1",
            Lang::En,
            1,
            1000,
            &mut Capture::default(),
        );
        assert!(json.contains("the program runs too long"), "{json}");
    }

    #[test]
    fn names_list_keywords_and_builtins_with_help() {
        let json = names_json();
        assert!(json.contains("\"разів\""));
        assert!(json.contains(
            "{\"uk\":\"коло\",\"en\":\"circle\",\"sig_uk\":\"коло(x, y, радіус)\",\"sig_en\":\"circle(x, y, r)\",\"doc_uk\":\"зафарбоване коло\",\"doc_en\":\"a filled circle\"}"
        ));
    }

    #[test]
    fn names_carry_the_version() {
        let tail = format!(",\"version\":\"{}\"}}", env!("CARGO_PKG_VERSION"));
        assert!(names_json().ends_with(&tail), "{}", names_json());
    }

    #[test]
    fn the_library_lists_sections_entries_and_colours() {
        let json = library_json();
        assert!(
            json.starts_with("{\"sections\":[{\"id\":\"basics\",\"uk\":\"Основи\",\"en\":\"Basics\",\"entries\":[{\"uk\":\"скажи\",\"en\":\"say\",\"form_uk\":\"скажи(…)\",\"form_en\":\"say(…)\","),
            "{json}"
        );
        assert_eq!(json.matches("\"form_uk\"").count(), 37);
        assert!(json.contains("\"ex_uk\":\"колір(\\\"червоний\\\")\\nколо(300, 200, 80)\""));
        assert!(json.ends_with("{\"uk\":\"коричневий\",\"en\":\"brown\",\"hex\":\"#8b5a2b\"}]}"));
    }
}
