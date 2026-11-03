//! `rok docs`: the pages it prints, the errors it gives, and the pages it
//! serves on localhost.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Command, Stdio};

fn rok(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_rok"))
        .args(args)
        .output()
        .expect("run rok")
}

#[test]
fn a_command_page_prints_as_markdown() {
    let output = rok(&["docs", "migrate", "up", "--print"]);
    assert_eq!(output.status.code(), Some(0));
    let page = String::from_utf8_lossy(&output.stdout);
    assert!(page.starts_with("# rok migrate up\n"), "{page}");
    assert!(page.contains("```text\nrok migrate up"), "{page}");
    assert!(page.contains("## Examples"), "{page}");
}

#[test]
fn the_index_prints_with_guides_and_commands() {
    let output = rok(&["docs", "--print"]);
    assert_eq!(output.status.code(), Some(0));
    let page = String::from_utf8_lossy(&output.stdout);
    assert!(
        page.contains("[Getting started](/guides/getting-started)"),
        "{page}"
    );
    assert!(page.contains("(/commands/db)"), "{page}");
}

#[test]
fn an_unknown_command_is_a_usage_error() {
    let output = rok(&["docs", "no", "such", "--print"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("no command `rok no such`"));
}

#[test]
fn docs_is_in_the_root_help() {
    let help = String::from_utf8_lossy(&rok(&["--help"]).stdout).into_owned();
    assert!(
        help.lines()
            .any(|line| line.trim_start().starts_with("docs")),
        "{help}"
    );
}

/// `GET path` on `address`, returning the status line and the body.
fn get(address: &str, path: &str) -> (String, String) {
    let mut stream = TcpStream::connect(address).expect("connect to rok docs");
    write!(stream, "GET {path} HTTP/1.1\r\nHost: {address}\r\n\r\n").expect("send");
    let mut response = String::new();
    stream.read_to_string(&mut response).expect("read");
    let status = response.lines().next().unwrap_or_default().to_owned();
    let body = response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body.to_owned())
        .unwrap_or_default();
    (status, body)
}

/// The server under test, stopped when the test ends however it ends: a
/// failed assertion must not leave `rok docs` running (on Windows it also
/// holds the test output's pipe and the binary open).
struct Server(std::process::Child);

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn it_serves_html_pages_on_localhost() {
    let mut server = Server(
        Command::new(env!("CARGO_BIN_EXE_rok"))
            .args(["docs", "db", "create", "--no-open", "--port", "0"])
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("run rok docs"),
    );
    let mut first = String::new();
    BufReader::new(server.0.stdout.take().expect("piped stdout"))
        .read_line(&mut first)
        .expect("the address line");
    let url = first.trim();
    assert!(
        url.starts_with("http://127.0.0.1:") && url.ends_with("/commands/db-create"),
        "{url}"
    );
    let address = url
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or_default()
        .to_owned();

    let (status, body) = get(&address, "/commands/db-create");
    assert!(status.contains("200"), "{status}");
    assert!(body.contains("<h1>rok db create</h1>"), "{body}");
    assert!(body.contains("<table>"), "{body}");

    let (status, _) = get(&address, "/guides/migrations");
    assert!(status.contains("200"), "{status}");
    let (status, _) = get(&address, "/commands/nope");
    assert!(status.contains("404"), "{status}");
}
