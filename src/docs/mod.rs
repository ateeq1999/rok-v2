//! `rok docs`: the documentation of the commands this `rok` answers, served
//! on localhost and opened in the browser, or printed as Markdown.
//!
//! The command pages come from the command tree `rok` links (rok-data's
//! commands with their examples, `tools`, `rok`'s own), so they describe
//! this binary's version; the guides are embedded with it.

use std::ffi::OsString;
use std::process::ExitCode;

use clap::{Arg, ArgAction};

mod pages;
mod serve;

/// The port `rok docs` serves on unless told otherwise.
const DEFAULT_PORT: u16 = 4730;

/// `rok docs`' grammar.
pub(crate) fn command() -> clap::Command {
    clap::Command::new("docs")
        .about("Open the documentation of every command in the browser (served on localhost)")
        .long_about(
            "Open the documentation of every command in the browser: a page per command \
             (its usage, options and examples, from this `rok`), and guides. The pages are \
             served on 127.0.0.1 until interrupted; --print writes the Markdown instead.",
        )
        .arg(
            Arg::new("command")
                .value_name("COMMAND")
                .num_args(0..)
                .help("Open this command's page, such as `db create`"),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .short('p')
                .value_name("PORT")
                .value_parser(clap::value_parser!(u16))
                .help("The port to serve on, or the first free one after it (default: 4730; 0 picks any)"),
        )
        .arg(
            Arg::new("no_open")
                .long("no-open")
                .short('n')
                .action(ArgAction::SetTrue)
                .help("Serve without opening the browser"),
        )
        .arg(
            Arg::new("print")
                .long("print")
                .action(ArgAction::SetTrue)
                .help("Print the page's Markdown on stdout instead of serving"),
        )
        .after_long_help(
            "Examples:\n  # Open the index in the browser\n  rok docs\n\n  \
             # Open `rok db create`'s page\n  rok docs db create\n\n  \
             # Print a command's page as Markdown\n  rok docs migrate up --print",
        )
}

/// Runs `rok docs` with `args` (the tokens after `docs`).
pub(crate) fn run(args: &[OsString]) -> ExitCode {
    let mut all = vec![OsString::from("rok docs")];
    all.extend(args.iter().cloned());
    let matches = match command().try_get_matches_from(all) {
        Ok(matches) => matches,
        Err(err) => {
            let _ = err.print();
            return ExitCode::from(if err.use_stderr() { 2 } else { 0 });
        }
    };
    let path: Vec<String> = matches
        .get_many::<String>("command")
        .map(|words| words.cloned().collect())
        .unwrap_or_default();
    let path: Vec<&str> = path.iter().map(String::as_str).collect();

    let (markdown, href) = if path.is_empty() {
        (pages::index(), "/".to_owned())
    } else {
        match pages::command_page(&path) {
            Some(page) => (page, pages::command_href(&path)),
            None => {
                eprintln!(
                    "error: no command `rok {}`\n\nRun `rok docs` for the commands.",
                    path.join(" ")
                );
                return ExitCode::from(2);
            }
        }
    };
    if matches.get_flag("print") {
        print!("{markdown}");
        return ExitCode::SUCCESS;
    }

    let port = matches
        .get_one::<u16>("port")
        .copied()
        .unwrap_or(DEFAULT_PORT);
    let listener = match serve::bind(port) {
        Ok(listener) => listener,
        Err(err) => {
            eprintln!("error: can't serve the documentation on 127.0.0.1:{port}: {err}");
            return ExitCode::FAILURE;
        }
    };
    let address = match listener.local_addr() {
        Ok(address) => address,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };
    let url = format!("http://{address}{href}");
    // The first line is the address, so a script can read it.
    println!("{url}");
    eprintln!("rok docs: serving on http://{address} (Ctrl-C to stop)");
    if !matches.get_flag("no_open") {
        open(&url);
    }
    serve::serve(&listener);
    ExitCode::SUCCESS
}

/// Opens `url` in the default browser; says so when it can't.
fn open(url: &str) {
    let opened = if cfg!(windows) {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", url])
            .status()
    } else if cfg!(target_os = "macos") {
        std::process::Command::new("open").arg(url).status()
    } else {
        std::process::Command::new("xdg-open").arg(url).status()
    };
    if !opened.is_ok_and(|status| status.success()) {
        eprintln!("rok docs: open {url} in your browser");
    }
}
