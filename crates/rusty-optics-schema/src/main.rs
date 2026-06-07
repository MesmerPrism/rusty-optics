//! Rusty Optics schema catalog command.

mod catalog;
mod cli;
mod error;

use std::process::ExitCode;

fn main() -> ExitCode {
    match cli::run(std::env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
