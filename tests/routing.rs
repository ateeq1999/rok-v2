//! `rok` routes each first word: rok-data's commands and `tools` to the CLIs
//! it links, any other word to `rok-<word>` from the `PATH`.

use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Output};

fn rok(args: &[&str]) -> Output {
    rok_with_path(args, None)
}

/// `rok <args>`, with `extra` put first on the `PATH` when given.
fn rok_with_path(args: &[&str], extra: Option<&Path>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rok"));
    command.args(args);
    if let Some(dir) = extra {
        let mut paths = vec![dir.to_owned()];
        paths.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        command.env(
            "PATH",
            std::env::join_paths(paths).unwrap_or_else(|_| OsString::new()),
        );
    }
    command.output().expect("run rok")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn help_lists_every_linked_command() {
    let output = rok(&["--help"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let help = stdout(&output);
    assert!(help.contains("Usage: rok"), "{help}");
    for command in [
        "init", "gen", "generate", "migrate", "db", "server", "schema", "sqlc", "tools",
    ] {
        assert!(
            help.lines()
                .any(|line| line.trim_start().starts_with(command)),
            "`{command}` missing from:\n{help}"
        );
    }
    assert!(help.contains("Installed"), "{help}");
}

#[test]
fn no_arguments_print_the_help() {
    let output = rok(&[]);
    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("Usage: rok"));
}

#[test]
fn version_is_the_umbrellas() {
    let output = rok(&["--version"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        stdout(&output).trim(),
        format!("rok {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn rok_datas_commands_say_rok() {
    for (args, usage) in [
        (&["db", "--help"][..], "Usage: rok db"),
        (&["migrate", "--help"][..], "Usage: rok migrate"),
        (&["sqlc", "--help"][..], "Usage: rok sqlc"),
    ] {
        let output = rok(args);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{args:?}: {}",
            stderr(&output)
        );
        assert!(
            stdout(&output).contains(usage),
            "{args:?}:\n{}",
            stdout(&output)
        );
    }
}

#[test]
fn a_usage_error_inside_rok_data_exits_two() {
    let output = rok(&["migrate", "--no-such-flag"]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
}

#[test]
fn tools_reaches_rok_tools() {
    let output = rok(&["tools", "--help"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(
        stdout(&output).contains("Usage: rok tools"),
        "{}",
        stdout(&output)
    );
}

#[test]
fn an_unknown_word_without_an_executable_is_a_usage_error_naming_it() {
    let output = rok(&["no-such-thing", "--flag"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        stderr(&output).contains("`rok-no-such-thing`"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn an_unknown_flag_is_a_usage_error() {
    let output = rok(&["--no-such-flag"]);
    assert_eq!(output.status.code(), Some(2));
}

/// A `rok-hello` on the `PATH` (a copy of `rok` itself) answers `rok hello`,
/// receives the remaining arguments, passes its exit code back, and is
/// listed under "Installed".
#[test]
fn a_rok_executable_on_the_path_answers_its_word() {
    let dir = tempfile::tempdir().expect("a temporary directory");
    let hello = dir
        .path()
        .join(format!("rok-hello{}", std::env::consts::EXE_SUFFIX));
    std::fs::copy(env!("CARGO_BIN_EXE_rok"), &hello).expect("copy rok");

    // `rok hello --version` runs `rok-hello --version`.
    let output = rok_with_path(&["hello", "--version"], Some(dir.path()));
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(stdout(&output).starts_with("rok "), "{}", stdout(&output));

    // Its own usage error (exit 2) comes back as `rok`'s.
    let output = rok_with_path(&["hello", "nope"], Some(dir.path()));
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("`rok-nope`"),
        "{}",
        stderr(&output)
    );

    let output = rok_with_path(&["--help"], Some(dir.path()));
    let help = stdout(&output);
    let installed = help
        .split("Installed")
        .nth(1)
        .unwrap_or_else(|| panic!("no Installed section:\n{help}"));
    assert!(
        installed.lines().any(|line| line.trim() == "hello"),
        "{help}"
    );
}
