use std::{env, fmt::Debug};

extern crate runner_common;
use runner_common::RunnerOptions;

pub enum CliError {
    UnexpectedArg(String),
}

impl Debug for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::UnexpectedArg(got) => write!(f, "unexpected argument \"{got}\""),
        }
    }
}

fn main() -> Result<(), CliError> {
    let mut opts = RunnerOptions::default();

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--verbose" | "-v" => opts.verbose = true,
            "--gdb" | "-d" => opts.gdb = true,
            _ => {
                return Err(CliError::UnexpectedArg(arg));
            }
        }
    }

    runner_common::run(&opts);

    Ok(())
}
