//! `rok` — one command for the rok toolchain (rok-data's workspace ADR
//! 0032).
//!
//! It owns no grammar of its own. The first word picks who answers:
//!
//! - a word of rok-data's CLI (`init`, `gen`, `migrate`, `db`, `server`,
//!   `schema`, `sqlc`, …, read from [`rok_data_cli::command`], so they never
//!   drift) goes to [`rok_data_cli::run`];
//! - `tools` goes to [`rok_tools_cli::run_from`];
//! - any other word `x` runs `rok-x` from the `PATH` with the remaining
//!   arguments, like `cargo` and `git` run their subcommands.
//!
//! Tokens reach the linked CLIs verbatim, displayed as `rok …`. Exit codes
//! follow ADR 0002: 0 for success and help, 2 for a usage error, 1 for a
//! runtime failure; a `rok-x` from the `PATH` passes its own through.

use std::ffi::{OsStr, OsString};
use std::process::ExitCode;

mod docs;
mod external;
mod help;

/// The word that reaches rok-tools.
const TOOLS: &str = "tools";

/// Run `rok` with `args` (the program name first, like
/// `std::env::args_os()`), returning the process exit code.
pub fn run<I, T>(args: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let mut args = args.into_iter().map(Into::into).skip(1);
    let Some(word) = args.next() else {
        print!("{}", help::render());
        return ExitCode::SUCCESS;
    };
    let rest: Vec<OsString> = args.collect();
    match word.to_str() {
        Some("-h" | "--help" | "help") if rest.is_empty() => {
            print!("{}", help::render());
            ExitCode::SUCCESS
        }
        Some("-V" | "--version") => {
            println!("rok {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("docs") => docs::run(&rest),
        Some(TOOLS) => {
            let code = rok_tools_cli::run_from(prepend("rok tools", &rest));
            ExitCode::from(u8::try_from(code).unwrap_or(1))
        }
        Some(name) if is_data_word(name) => {
            // rok-data's CLI takes `String`s (and names itself after the
            // first one, so its help says `rok …`).
            let mut forwarded = vec!["rok".to_owned(), name.to_owned()];
            forwarded.extend(rest.iter().map(|arg| shown(arg)));
            rok_data_cli::run(forwarded)
        }
        Some(name) if name.starts_with('-') => {
            eprintln!("error: unexpected argument `{name}`\n\nRun `rok --help` for the commands.");
            ExitCode::from(2)
        }
        _ => external::run(&word, &rest),
    }
}

/// `program` followed by `args`, as a CLI's argument list.
fn prepend(program: &str, args: &[OsString]) -> Vec<OsString> {
    std::iter::once(OsString::from(program))
        .chain(args.iter().cloned())
        .collect()
}

/// Whether `word` is one of rok-data's commands or their aliases.
fn is_data_word(word: &str) -> bool {
    rok_data_cli::command().get_subcommands().any(|command| {
        command.get_name() == word || command.get_all_aliases().any(|alias| alias == word)
    })
}

/// The commands `rok` links, by name, for help and the PATH listing.
fn linked_words() -> Vec<String> {
    rok_data_cli::command()
        .get_subcommands()
        .map(|command| command.get_name().to_owned())
        .chain([TOOLS.to_owned(), "docs".to_owned()])
        .collect()
}

/// An `OsStr` as text for messages.
fn shown(text: &OsStr) -> String {
    text.to_string_lossy().into_owned()
}
