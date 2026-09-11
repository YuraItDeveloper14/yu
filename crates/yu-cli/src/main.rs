mod color;

use std::io::{self, BufRead, IsTerminal, Write};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use color::Paint;
use yu_core::{Host, Lang, Options, Session, Value};

/// Deep recursion needs more than the 1 MB main-thread stack Windows gives.
const STACK: usize = 256 * 1024 * 1024;

struct Terminal;

impl Host for Terminal {
    fn print(&mut self, line: &str) {
        println!("{line}");
    }

    fn ask(&mut self, prompt: &str) -> Option<String> {
        print!("{prompt} ");
        io::stdout().flush().ok();
        let mut line = String::new();
        match io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => None,
            Ok(_) => Some(line),
        }
    }
}

enum Command {
    Repl,
    Run(String),
    Help,
    Version,
}

struct Args {
    lang: Lang,
    color: bool,
    command: Command,
}

fn usage(lang: Lang) -> &'static str {
    lang.pick(
        "Yu — мова програмування українською та англійською\n\n\
         Використання:\n  yu                  інтерактивний режим\n  yu файл.yu          запустити програму\n  \
         yu run файл.yu      те саме\n  --lang en           помилки англійською\n  --no-color          без кольорів\n  \
         --version           версія",
        "Yu — a programming language in Ukrainian and English\n\n\
         Usage:\n  yu                  interactive mode\n  yu file.yu          run a program\n  \
         yu run file.yu      the same\n  --lang uk           messages in Ukrainian\n  --no-color          no colours\n  \
         --version           version",
    )
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Args, String> {
    let (mut lang, mut color, mut help, mut version) = (Lang::Uk, true, false, false);
    let mut files = Vec::new();
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--lang" => {
                let code = args.next().unwrap_or_default();
                lang = Lang::from_code(&code)
                    .ok_or_else(|| format!("--lang: uk / en, не «{code}»"))?;
            }
            "--no-color" => color = false,
            "-h" | "--help" => help = true,
            "-V" | "--version" => version = true,
            "run" if files.is_empty() => {}
            other if other.starts_with('-') => {
                return Err(format!(
                    "{} {other}",
                    lang.pick("невідомий параметр", "unknown option")
                ));
            }
            other => files.push(other.to_string()),
        }
    }
    let command = if help {
        Command::Help
    } else if version {
        Command::Version
    } else {
        match files.len() {
            0 => Command::Repl,
            1 => Command::Run(files.remove(0)),
            _ => {
                return Err(lang
                    .pick("можна запускати один файл", "one file at a time")
                    .into())
            }
        }
    };
    Ok(Args {
        lang,
        color,
        command,
    })
}

fn seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
        | 1
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("yu: {message}\n\n{}", usage(Lang::Uk));
            return ExitCode::from(2);
        }
    };
    let paint = Paint::new(
        args.color && io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none(),
    );
    let opts = Options {
        lang: args.lang,
        seed: seed(),
        ..Options::default()
    };
    match args.command {
        Command::Help => {
            println!("{}", usage(args.lang));
            ExitCode::SUCCESS
        }
        Command::Version => {
            println!("yu {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Command::Run(path) => run_file(&path, opts, paint),
        Command::Repl => {
            let repl = std::thread::Builder::new()
                .stack_size(STACK)
                .spawn(move || repl(opts, paint));
            repl.expect("start the REPL thread").join().ok();
            ExitCode::SUCCESS
        }
    }
}

fn run_file(path: &str, opts: Options, paint: Paint) -> ExitCode {
    let src = match std::fs::read_to_string(path) {
        Ok(src) => src,
        Err(e) => {
            eprintln!(
                "yu: {} {path}: {e}",
                opts.lang.pick("не вдалося прочитати", "can't read")
            );
            return ExitCode::from(2);
        }
    };
    let worker = std::thread::Builder::new()
        .stack_size(STACK)
        .spawn(move || {
            let result = yu_core::run(&src, &mut Terminal, opts);
            result.map_err(|e| e.render(&src, opts.lang))
        });
    match worker.expect("start the program thread").join() {
        Ok(Ok(())) => ExitCode::SUCCESS,
        Ok(Err(rendered)) => {
            eprintln!("{}", paint.diagnostic(&rendered));
            ExitCode::from(1)
        }
        Err(_) => ExitCode::from(101),
    }
}

/// A buffer whose first line ends with `:` is a block; it waits for an empty line.
fn opens_block(buffer: &str) -> bool {
    buffer
        .lines()
        .next()
        .is_some_and(|l| l.split('#').next().unwrap_or("").trim_end().ends_with(':'))
}

fn repl(opts: Options, paint: Paint) {
    let greeting = opts.lang.pick(
        "Yu {v} — пиши українською або англійською. Вийти: «вийти» або Ctrl+C.",
        "Yu {v} — write in Ukrainian or English. Quit: 'exit' or Ctrl+C.",
    );
    println!("{}", greeting.replace("{v}", env!("CARGO_PKG_VERSION")));
    let mut session = Session::new(opts);
    let mut buffer = String::new();
    loop {
        print!("{}", if buffer.is_empty() { "yu> " } else { "... " });
        io::stdout().flush().ok();
        let mut line = String::new();
        if io::stdin().lock().read_line(&mut line).unwrap_or(0) == 0 {
            println!();
            break;
        }
        let line = line.trim_end();
        if buffer.is_empty() && matches!(line.trim(), "вийти" | "exit" | "quit") {
            break;
        }
        buffer.push_str(line);
        buffer.push('\n');
        if opens_block(&buffer) && !line.is_empty() {
            continue;
        }
        let src = std::mem::take(&mut buffer);
        if src.trim().is_empty() {
            continue;
        }
        match session.run(&src, &mut Terminal) {
            Ok(Some(value)) if value != Value::Nothing => println!("{}", value.display(opts.lang)),
            Ok(_) => {}
            Err(e) => eprintln!("{}", paint.diagnostic(&e.render(&src, opts.lang))),
        }
    }
}
