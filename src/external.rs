//! `rok <name> …` for a name nothing links: `rok-<name>` from the `PATH`,
//! the way `cargo` and `git` find their subcommands.

use std::ffi::{OsStr, OsString};
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use crate::shown;

/// The standalone binaries of the CLIs `rok` links: on the `PATH` they
/// would only repeat a linked command, so the listing leaves them out.
const LINKED_BINARIES: [&str; 2] = ["rok-data", "rok-tools"];

/// Runs `rok-<name> <args>` from the `PATH` and returns its exit code; no
/// such executable is a usage error (exit 2) naming the one looked for.
pub(crate) fn run(name: &OsStr, args: &[OsString]) -> ExitCode {
    let wanted = format!("rok-{}", shown(name));
    let Some(path) = find(&wanted) else {
        eprintln!(
            "error: unrecognized command `{}`: no `{wanted}` on the PATH\n\n\
             Run `rok --help` for the commands.",
            shown(name)
        );
        return ExitCode::from(2);
    };
    match Command::new(&path).args(args).status() {
        Ok(status) => match status.code() {
            Some(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
            // Ended by a signal.
            None => ExitCode::FAILURE,
        },
        Err(err) => {
            eprintln!("error: can't run {}: {err}", path.display());
            ExitCode::FAILURE
        }
    }
}

/// The first `<file>` (with the platform's executable suffix) in a `PATH`
/// directory.
fn find(file: &str) -> Option<PathBuf> {
    let name = format!("{file}{}", std::env::consts::EXE_SUFFIX);
    std::env::split_paths(&std::env::var_os("PATH")?)
        .map(|dir| dir.join(&name))
        .find(|candidate| is_executable(candidate))
}

/// The names `rok` would run from the `PATH` (`hello` for `rok-hello`),
/// sorted, without the linked CLIs' own binaries or a name a linked
/// command already answers.
pub(crate) fn installed(linked: &[String]) -> Vec<String> {
    let Some(path) = std::env::var_os("PATH") else {
        return Vec::new();
    };
    let suffix = std::env::consts::EXE_SUFFIX;
    let mut names: Vec<String> = std::env::split_paths(&path)
        .filter_map(|dir| std::fs::read_dir(dir).ok())
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| is_executable(&entry.path()))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter_map(|file| {
            let stem = file.strip_suffix(suffix).unwrap_or(&file).to_owned();
            (!LINKED_BINARIES.contains(&stem.as_str()))
                .then(|| stem.strip_prefix("rok-").map(str::to_owned))
                .flatten()
        })
        .filter(|name| !name.is_empty() && !linked.contains(name))
        .collect();
    names.sort();
    names.dedup();
    names
}

#[cfg(unix)]
fn is_executable(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    path.metadata()
        .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &std::path::Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case(std::env::consts::EXE_EXTENSION))
        && path.is_file()
}
