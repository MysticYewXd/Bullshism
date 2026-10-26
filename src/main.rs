//! Bullshism: a launcher full of carefully engineered nonsense.

mod apps;
mod cli;
mod launcher;
mod liar;
mod parser;

fn main() -> std::process::ExitCode {
    cli::main()
}
