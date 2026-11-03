//! `rok update` against a release server of the test's own: the release it
//! names, the archive and checksum it serves, and what lands where. The
//! new binary goes to `ROK_UPDATE_DESTINATION`, never over the real `rok`.

use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::Path;
use std::process::{Command, Output};

use sha2::{Digest, Sha256};

const NEW_BINARY: &[u8] = b"the new rok";

/// An archive holding `rok-9.9.9-x/rok` (or `rok.exe` in a zip).
fn archive(zip_format: bool) -> Vec<u8> {
    if zip_format {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        zip.start_file(
            "rok-9.9.9-x/rok.exe",
            zip::write::SimpleFileOptions::default(),
        )
        .expect("zip entry");
        zip.write_all(NEW_BINARY).expect("zip data");
        zip.finish().expect("zip").into_inner()
    } else {
        let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        {
            let mut tar = tar::Builder::new(&mut gz);
            let mut header = tar::Header::new_gnu();
            header.set_size(NEW_BINARY.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            tar.append_data(&mut header, "rok-9.9.9-x/rok", NEW_BINARY)
                .expect("tar entry");
            tar.finish().expect("tar");
        }
        gz.finish().expect("gzip")
    }
}

fn hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Serves the release `v9.9.9` until the test ends; `good_checksum: false`
/// serves a checksum that doesn't match. Returns `http://127.0.0.1:<port>`.
fn release_server(good_checksum: bool) -> String {
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind");
    let address = listener.local_addr().expect("address");
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut line = String::new();
            let _ = BufReader::new(&stream).read_line(&mut line);
            let path = line.split_whitespace().nth(1).unwrap_or("/").to_owned();
            let body: Vec<u8> = if path == "/latest" {
                br#"{"tag_name": "v9.9.9"}"#.to_vec()
            } else if let Some(name) = path.strip_prefix("/downloads/v9.9.9/") {
                if let Some(archive_name) = name.strip_suffix(".sha256") {
                    let digest = if good_checksum {
                        hex(&archive(archive_name.ends_with(".zip")))
                    } else {
                        "0".repeat(64)
                    };
                    format!("{digest}  {archive_name}\n").into_bytes()
                } else {
                    archive(name.ends_with(".zip"))
                }
            } else {
                let _ = write!(
                    stream,
                    "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n"
                );
                continue;
            };
            let _ = write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(&body);
        }
    });
    format!("http://{address}")
}

fn update(server: &str, destination: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rok"))
        .arg("update")
        .args(args)
        .env("ROK_UPDATE_API", format!("{server}/latest"))
        .env("ROK_UPDATE_DOWNLOADS", format!("{server}/downloads"))
        .env("ROK_UPDATE_DESTINATION", destination)
        .output()
        .expect("run rok update")
}

#[test]
fn check_reports_a_newer_release_and_installs_nothing() {
    let server = release_server(true);
    let dir = tempfile::tempdir().expect("temp dir");
    let destination = dir.path().join("rok-new");
    let output = update(&server, &destination, &["--check", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("one JSON object");
    assert_eq!(report["latest"], "9.9.9");
    assert_eq!(report["update_available"], true);
    assert_eq!(report["action"], "none");
    assert!(!destination.exists());
}

#[test]
fn it_downloads_verifies_and_installs_the_release() {
    let server = release_server(true);
    let dir = tempfile::tempdir().expect("temp dir");
    let destination = dir.path().join("rok-new");
    let output = update(&server, &destination, &[]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains(&format!("rok {} → 9.9.9", env!("CARGO_PKG_VERSION"))),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(
        std::fs::read(&destination).expect("the installed binary"),
        NEW_BINARY
    );
}

#[test]
fn a_checksum_mismatch_installs_nothing() {
    let server = release_server(false);
    let dir = tempfile::tempdir().expect("temp dir");
    let destination = dir.path().join("rok-new");
    let output = update(&server, &destination, &[]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("nothing was installed"), "{stderr}");
    assert!(
        !destination.exists(),
        "a binary was installed despite the bad checksum"
    );
}

#[test]
fn the_running_version_is_up_to_date_without_force() {
    let server = release_server(true);
    let dir = tempfile::tempdir().expect("temp dir");
    let destination = dir.path().join("rok-new");
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));
    let output = update(&server, &destination, &["--version", &version]);
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("is up to date"));
    assert!(!destination.exists());
}

#[test]
fn a_malformed_version_is_a_usage_error() {
    let output = Command::new(env!("CARGO_BIN_EXE_rok"))
        .args(["update", "--version", "latest-please"])
        .output()
        .expect("run rok update");
    assert_eq!(output.status.code(), Some(2));
}
