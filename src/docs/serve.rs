//! `rok docs`' web server: the pages as HTML on `127.0.0.1`, for this
//! machine only. One request at a time; it runs until interrupted.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

use super::pages;

/// The first free port from `port` up (ten tries), on the loopback
/// interface; `0` lets the system pick one.
pub(crate) fn bind(port: u16) -> std::io::Result<TcpListener> {
    if port == 0 {
        return TcpListener::bind(("127.0.0.1", 0));
    }
    let mut last = None;
    for candidate in port..port.saturating_add(10) {
        match TcpListener::bind(("127.0.0.1", candidate)) {
            Ok(listener) => return Ok(listener),
            Err(err) => last = Some(err),
        }
    }
    Err(last.unwrap_or_else(|| std::io::Error::other("no port to try")))
}

/// Answers requests on `listener` until the process ends.
pub(crate) fn serve(listener: &TcpListener) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(err) = answer(stream) {
                    eprintln!("rok docs: {err}");
                }
            }
            Err(err) => eprintln!("rok docs: {err}"),
        }
    }
}

fn answer(mut stream: TcpStream) -> std::io::Result<()> {
    let mut request_line = String::new();
    BufReader::new(&stream).read_line(&mut request_line)?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or("/");
    let path = target.split(['?', '#']).next().unwrap_or("/");

    let (status, title, markdown) = if method != "GET" {
        (
            "405 Method Not Allowed",
            "Not allowed".to_owned(),
            "# Only GET\n".to_owned(),
        )
    } else {
        route(path)
    };
    let body = page(&title, &markdown);
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\n\
         Content-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )?;
    stream.flush()
}

/// The status, title and Markdown for `path`.
fn route(path: &str) -> (&'static str, String, String) {
    let found = if path == "/" {
        Some(("rok".to_owned(), pages::index()))
    } else if let Some(slug) = path.strip_prefix("/commands/") {
        pages::path_of(slug).and_then(|path| {
            let path: Vec<&str> = path.iter().map(String::as_str).collect();
            pages::command_page(&path).map(|page| (format!("rok {}", path.join(" ")), page))
        })
    } else if let Some(slug) = path.strip_prefix("/guides/") {
        pages::guide(slug).map(|guide| (guide.title.to_owned(), guide.markdown.to_owned()))
    } else {
        None
    };
    match found {
        Some((title, markdown)) => ("200 OK", title, markdown),
        None => (
            "404 Not Found",
            "Not found".to_owned(),
            format!("# Not found\n\nNo page at `{path}`. [Back to the index](/).\n"),
        ),
    }
}

/// `markdown` as a whole HTML page.
fn page(title: &str, markdown: &str) -> String {
    let mut body = String::new();
    let options = pulldown_cmark::Options::ENABLE_TABLES;
    pulldown_cmark::html::push_html(
        &mut body,
        pulldown_cmark::Parser::new_ext(markdown, options),
    );
    format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <title>{} · rok docs</title><style>{STYLE}</style></head>\
         <body><header><a href=\"/\">rok docs</a></header><main>{body}</main></body></html>\n",
        escape(title)
    )
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

const STYLE: &str = "\
:root{--bg:#fafaf9;--fg:#1c1917;--muted:#57534e;--line:#e7e5e4;--code:#f5f5f4;--link:#0f766e}\
@media (prefers-color-scheme:dark){:root{--bg:#1c1917;--fg:#f5f5f4;--muted:#a8a29e;--line:#44403c;--code:#292524;--link:#5eead4}}\
*{box-sizing:border-box}body{margin:0;background:var(--bg);color:var(--fg);\
font:16px/1.6 system-ui,-apple-system,Segoe UI,sans-serif}\
header{border-bottom:1px solid var(--line);padding:12px 16px}header a{font-weight:600;color:var(--fg);text-decoration:none}\
main{max-width:860px;margin:0 auto;padding:16px}a{color:var(--link)}\
h1{font-size:1.8rem;margin:.6em 0 .4em}h2{font-size:1.25rem;margin:1.6em 0 .5em;border-bottom:1px solid var(--line);padding-bottom:.2em}\
code{font-family:ui-monospace,SFMono-Regular,Consolas,monospace;font-size:.9em;background:var(--code);padding:.1em .3em;border-radius:4px}\
pre{background:var(--code);padding:12px;border-radius:6px;overflow-x:auto}pre code{padding:0;background:none}\
table{border-collapse:collapse;width:100%;display:block;overflow-x:auto}th,td{border:1px solid var(--line);padding:6px 10px;text-align:left;vertical-align:top}\
th{background:var(--code)}li{margin:.2em 0}p{color:var(--fg)}";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_pages_are_found_and_others_are_not() {
        assert_eq!(route("/").0, "200 OK");
        assert_eq!(route("/commands/db-create").0, "200 OK");
        assert_eq!(route("/guides/getting-started").0, "200 OK");
        assert_eq!(route("/commands/nope").0, "404 Not Found");
        assert_eq!(route("/etc/passwd").0, "404 Not Found");
    }

    #[test]
    fn markdown_becomes_html_with_tables() {
        let html = page("t", "# Hi\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n");
        assert!(html.contains("<h1>Hi</h1>"), "{html}");
        assert!(html.contains("<table>"), "{html}");
    }

    #[test]
    fn titles_are_escaped() {
        assert!(page("<x>", "").contains("<title>&lt;x&gt; · rok docs</title>"));
    }
}
