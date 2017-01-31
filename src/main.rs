// Copyright (C) 2017 Stephane Raux. Distributed under the MIT license.

#![deny(warnings)]

extern crate everust;
extern crate rustyline;

use rustyline::Editor;
use rustyline::error::ReadlineError;
use std::error::Error;
use std::fmt::{Display, self};
use std::io::{self, stderr, stdout, Write};

#[derive(Debug)]
enum ProgError {
    FailedToReadLine(ReadlineError),
    StderrErr(io::Error),
    StdoutErr(io::Error),
}

impl Error for ProgError {
    fn cause(&self) -> Option<&Error> {
        match *self {
            ProgError::FailedToReadLine(ref e) => Some(e),
            ProgError::StderrErr(ref e) => Some(e),
            ProgError::StdoutErr(ref e) => Some(e),
        }
    }

    fn description(&self) -> &str {
        match *self {
            ProgError::FailedToReadLine(_) => "Failed to read line",
            ProgError::StderrErr(_) => "stderr error",
            ProgError::StdoutErr(_) => "stdout error",
        }
    }
}

impl Display for ProgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

fn evaluate(code: &str) -> Result<(), ProgError> {
    match everust::eval(code) {
        Ok(s) => writeln!(stdout(), "{}", s).map_err(ProgError::StdoutErr),
        Err(e) => print_error_trace(&e).map_err(ProgError::StderrErr),
    }
}

fn print_error_trace(e: &Error) -> io::Result<()> {
    writeln!(stderr(), "{}", e)?;
    let mut e = e;
    while let Some(cause) = e.cause() {
        writeln!(stderr(), "Because: {}", cause)?;
        e = cause;
    }
    Ok(())
}

fn run_repl() -> Result<(), ProgError> {
    let mut editor = Editor::<()>::new();
    loop {
        match editor.readline("> ") {
            Ok(line) => if line == "exit" {
                return Ok(())
            } else {
                editor.add_history_entry(&line);
                evaluate(&line)?;
            },
            Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => {
                return Ok(())
            }
            Err(e) => return Err(ProgError::FailedToReadLine(e)),
        }
    }
}

fn to_exit_code(r: Result<(), ProgError>) -> i32 {
    let e = match r {
        Ok(()) => return 0,
        Err(e) => e,
    };
    let log = match e {
        ProgError::StdoutErr(ref e) => e.kind() != io::ErrorKind::BrokenPipe,
        ProgError::StderrErr(_) => false,
        _ => true,
    };
    if log {print_error_trace(&e).unwrap_or(());}
    1
}

fn main() {
    std::process::exit(to_exit_code(run_repl()))
}
