//! `rok update`: installs the latest (or a named) release of `rok` over the
//! running one. rok-data's and rok-tools' commands are linked into `rok`,
//! so this updates them too.
//!
//! A `rok` installed with `cargo install` updates through Cargo, so
//! Cargo's records stay right; any other `rok` downloads its platform's
//! release archive, checks it against the release's SHA-256 file, and
//! replaces itself.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Arg, ArgAction};

mod release;

use release::Version;

/// `rok update`'s grammar.
pub(crate) fn command() -> clap::Command {
    clap::Command::new("update")
        .about("Install the latest release of rok over this one")
        .long_about(
            "Install the latest release of rok (or --version's) over this one; rok-data's and \
             rok-tools' commands come with it. A rok installed with `cargo install` updates with \
             Cargo; any other downloads its platform's release archive from GitHub, checks its \
             SHA-256, and replaces the running binary.",
        )
        .arg(
            Arg::new("check")
                .long("check")
                .short('c')
                .action(ArgAction::SetTrue)
                .help("Only say whether a newer release exists; install nothing"),
        )
        .arg(
            Arg::new("version")
                .long("version")
                .short('v')
                .value_name("VERSION")
                .value_parser(|text: &str| {
                    Version::parse(text)
                        .map(|_| text.to_owned())
                        .ok_or_else(|| format!("`{text}` is not a release version (vX.Y.Z)"))
                })
                .help("Install this release (vX.Y.Z) instead of the latest"),
        )
        .arg(
            Arg::new("force")
                .long("force")
                .short('f')
                .action(ArgAction::SetTrue)
                .help("Install even when that version is already the one running"),
        )
        .arg(
            Arg::new("json")
                .long("json")
                .action(ArgAction::SetTrue)
                .help("Print one JSON object: the current and latest versions, and what was done"),
        )
        .after_long_help(
            "Examples:\n  # Is there a newer rok?\n  rok update --check\n\n  \
             # Install the latest release\n  rok update\n\n  \
             # Install one release\n  rok update --version v0.7.1",
        )
}

/// Runs `rok update` with `args` (the tokens after `update`).
pub(crate) fn run(args: &[OsString]) -> ExitCode {
    let mut all = vec![OsString::from("rok update")];
    all.extend(args.iter().cloned());
    let matches = match command().try_get_matches_from(all) {
        Ok(matches) => matches,
        Err(err) => {
            let _ = err.print();
            return ExitCode::from(if err.use_stderr() { 2 } else { 0 });
        }
    };
    let json = matches.get_flag("json");
    match update(
        matches.get_one::<String>("version").map(String::as_str),
        matches.get_flag("check"),
        matches.get_flag("force"),
        json,
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            if json {
                println!("{}", serde_json::json!({ "error": message }));
            } else {
                eprintln!("error: {message}");
            }
            ExitCode::FAILURE
        }
    }
}

fn update(wanted: Option<&str>, check: bool, force: bool, json: bool) -> Result<(), String> {
    let current = release::current();
    let target = match wanted {
        Some(text) => Version::parse(text)
            .ok_or_else(|| format!("`{text}` is not a release version (vX.Y.Z)"))?,
        None => release::latest()?,
    };
    let newer = target > current;
    let say = |action: &str, text: &str| {
        if json {
            println!(
                "{}",
                serde_json::json!({
                    "current": current.to_string(),
                    "latest": target.to_string(),
                    "update_available": newer,
                    "action": action,
                })
            );
        } else {
            println!("{text}");
        }
    };

    if check {
        if newer {
            say(
                "none",
                &format!("rok {target} is available (this is {current}); run `rok update`"),
            );
        } else {
            say("none", &format!("rok {current} is up to date"));
        }
        return Ok(());
    }
    if target == current && !force {
        say("none", &format!("rok {current} is up to date"));
        return Ok(());
    }

    let destination = std::env::var_os("ROK_UPDATE_DESTINATION").map(PathBuf::from);
    if destination.is_none() && installed_by_cargo() {
        cargo_install(target)?;
        say(
            "cargo-install",
            &format!("rok {current} → {target} (cargo install rok-cli)"),
        );
        return Ok(());
    }

    let name = release::asset_name(target, env!("ROK_TARGET")).ok_or_else(|| {
        format!(
            "no release archive is built for {}; run `cargo install rok-cli --version {target}`",
            env!("ROK_TARGET")
        )
    })?;
    let archive = release::download(target, &name)?;
    let checksum = release::download(target, &format!("{name}.sha256"))?;
    release::verify(&archive, &String::from_utf8_lossy(&checksum), &name)?;
    let binary = release::extract(&archive, &name)?;
    install(&binary, destination.as_deref())?;
    say("replaced", &format!("rok {current} → {target}"));
    Ok(())
}

/// Whether the running `rok` sits in Cargo's `bin` directory.
fn installed_by_cargo() -> bool {
    let Ok(exe) = std::env::current_exe() else {
        return false;
    };
    let cargo_home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
                .map(|home| PathBuf::from(home).join(".cargo"))
        });
    let (Some(home), Some(dir)) = (cargo_home, exe.parent()) else {
        return false;
    };
    let canonical = |path: &Path| path.canonicalize().unwrap_or_else(|_| path.to_owned());
    canonical(dir) == canonical(&home.join("bin"))
}

fn cargo_install(version: Version) -> Result<(), String> {
    let status = std::process::Command::new("cargo")
        .args([
            "install",
            "rok-cli",
            "--locked",
            "--version",
            &version.to_string(),
        ])
        .status()
        .map_err(|err| format!("can't run cargo: {err}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "`cargo install rok-cli --version {version}` failed ({status})"
        ))
    }
}

/// Writes `binary` over the running `rok`, or to `destination` when given.
fn install(binary: &[u8], destination: Option<&Path>) -> Result<(), String> {
    let staged = std::env::temp_dir().join(format!("rok-update-{}", std::process::id()));
    std::fs::write(&staged, binary).map_err(|err| format!("can't stage the new rok: {err}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&staged, std::fs::Permissions::from_mode(0o755));
    }
    let result = match destination {
        Some(path) => std::fs::copy(&staged, path)
            .map(|_| ())
            .map_err(|err| format!("can't write {}: {err}", path.display())),
        None => self_replace::self_replace(&staged)
            .map_err(|err| format!("can't replace the running rok: {err}")),
    };
    let _ = std::fs::remove_file(&staged);
    result
}
