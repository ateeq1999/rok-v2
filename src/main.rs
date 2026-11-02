//! The `rok` binary: everything is in the library, so the tests drive the
//! same code.

use std::process::ExitCode;

fn main() -> ExitCode {
    rok_cli::run(std::env::args_os())
}
