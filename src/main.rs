//! Bullshism: a carefully engineered calculator that is always wrong.

mod liar;
mod parser;

use liar::{Rng, VERDICTS, format_number, lie};
use std::io::{self, BufRead, IsTerminal, Write};
use std::process::ExitCode;

const HELP: &str = "\
Bullshism - the calculator that is confidently wrong

USAGE:
    bullshism [OPTIONS] [EXPRESSION...]

    With an expression, prints one (wrong) answer and exits.
    Without one, starts an interactive session.

OPTIONS:
    -c, --confess     Also show the real answer and how it was ruined
    -s, --seed <N>    Use a fixed seed, so the same lies come out every time
    -h, --help        Show this help
    -V, --version     Show the version

SUPPORTED MATHS (computed honestly, reported dishonestly):
    + - * / % ^ !     e.g. 2+2, 2^10, 5!, 10 % 3
    ( ) and 2(3+4)    brackets and implicit multiplication
    sqrt cbrt abs sin cos tan asin acos atan ln log exp floor ceil round
    pi e tau phi ans  constants, and ans for the last answer

SESSION COMMANDS:
    help              this text
    confess           toggle confess mode
    quit / exit       leave (Ctrl+D works too)";

struct Style {
    on: bool,
}

impl Style {
    fn paint(&self, code: &str, text: &str) -> String {
        if self.on {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
    fn answer(&self, t: &str) -> String {
        self.paint("1;32", t)
    }
    fn dim(&self, t: &str) -> String {
        self.paint("2", t)
    }
    fn error(&self, t: &str) -> String {
        self.paint("1;31", t)
    }
    fn title(&self, t: &str) -> String {
        self.paint("1;35", t)
    }
}

struct Session {
    rng: Rng,
    confess: bool,
    ans: f64,
    style: Style,
}

impl Session {
    /// Calculate one line. Returns false if it was an error.
    fn calculate(&mut self, input: &str) -> bool {
        match parser::evaluate(input, self.ans) {
            Ok(truth) => {
                let (answer, strategy) = lie(truth, &mut self.rng);
                let verdict = *self.rng.pick(&VERDICTS);
                println!(
                    "{} {}",
                    self.style.answer(&format!("= {}", format_number(answer))),
                    self.style.dim(verdict)
                );
                if self.confess {
                    println!(
                        "  {}",
                        self.style.dim(&format!(
                            "(really {}; {})",
                            format_number(truth),
                            strategy.name()
                        ))
                    );
                }
                // ans remembers what we said, not what was true. Lies compound.
                self.ans = answer;
                true
            }
            Err(e) => {
                eprintln!("{}", self.style.error(&format!("error: {e}")));
                false
            }
        }
    }

    fn repl(&mut self) {
        println!(
            "{} {}",
            self.style.title("Bullshism"),
            self.style
                .dim("- the calculator you can't trust. Type 'help' or 'quit'.")
        );
        let stdin = io::stdin();
        let mut line = String::new();
        loop {
            print!("{} ", self.style.title(">"));
            let _ = io::stdout().flush();
            line.clear();
            match stdin.lock().read_line(&mut line) {
                Ok(0) => {
                    println!();
                    break;
                }
                Ok(_) => {}
                Err(e) => {
                    eprintln!("{}", self.style.error(&format!("error: {e}")));
                    break;
                }
            }
            let input = line.trim();
            match input.to_lowercase().as_str() {
                "" => continue,
                "quit" | "exit" | "q" => break,
                "help" | "?" => println!("{HELP}"),
                "confess" => {
                    self.confess = !self.confess;
                    let state = if self.confess { "on" } else { "off" };
                    println!("{}", self.style.dim(&format!("confess mode {state}")));
                }
                _ => {
                    self.calculate(input);
                }
            }
        }
        println!(
            "{}",
            self.style.dim("Goodbye. Everything I told you was true.")
        );
    }
}

fn main() -> ExitCode {
    let mut confess = false;
    let mut seed: Option<u64> = None;
    let mut words: Vec<String> = Vec::new();

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("{HELP}");
                return ExitCode::SUCCESS;
            }
            "-V" | "--version" => {
                println!("bullshism {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "-c" | "--confess" => confess = true,
            "-s" | "--seed" => match args.next().and_then(|v| v.parse().ok()) {
                Some(n) => seed = Some(n),
                None => {
                    eprintln!("error: --seed needs a whole number");
                    return ExitCode::from(2);
                }
            },
            // Anything else is part of the expression (so "-3+1" still works).
            _ => words.push(arg),
        }
    }

    let mut session = Session {
        rng: seed.map(Rng::new).unwrap_or_else(Rng::from_time),
        confess,
        ans: 0.0,
        style: Style {
            on: io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none(),
        },
    };

    if words.is_empty() {
        session.repl();
        ExitCode::SUCCESS
    } else if session.calculate(&words.join(" ")) {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
