//! What `cargo package` ships: the build script, and every file the
//! sources embed. A missing one builds here but fails `cargo install
//! rok-cli` (rok-data 0.6.0's release stopped on exactly that).

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn packaged() -> Vec<String> {
    let output = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()))
        .args(["package", "--list", "--allow-dirty", "--offline"])
        .current_dir(root())
        .output()
        .expect("run cargo package --list");
    assert!(
        output.status.success(),
        "cargo package --list failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.replace('\\', "/"))
        .collect()
}

fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("read the sources") {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            sources(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn the_package_ships_the_build_script_and_every_embedded_file() {
    let files = packaged();
    assert!(
        files.contains(&"build.rs".to_owned()),
        "build.rs is not packaged"
    );
    let mut all = Vec::new();
    sources(&root().join("src"), &mut all);
    let mut missing = Vec::new();
    for source in &all {
        let text = std::fs::read_to_string(source).expect("read a source");
        for (at, _) in text.match_indices("include_str!(\"") {
            let rest = &text[at + "include_str!(\"".len()..];
            let Some(end) = rest.find('"') else { continue };
            let embedded = source.parent().expect("a directory").join(&rest[..end]);
            let relative = embedded
                .strip_prefix(root())
                .expect("inside the crate")
                .to_string_lossy()
                .replace('\\', "/");
            if !files.contains(&relative) {
                missing.push(relative);
            }
        }
    }
    assert!(
        missing.is_empty(),
        "embedded but not packaged (add them to Cargo.toml's `include`): {missing:?}"
    );
}
