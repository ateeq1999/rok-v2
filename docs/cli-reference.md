# `rok` — the root grammar

`rok` owns only its first word. Everything after it belongs to the CLI
that answers, whose reference is the contract for it:

| First word | Answered by | Its reference |
| --- | --- | --- |
| `init`, `gen`, `generate`, `migrate`, `db`, `server`, `schema`, `sqlc` (and their aliases) | `rok-data-cli`, linked | [rok-data's `docs/cli-reference.md`](https://github.com/ateeq1999/rok-data/blob/main/docs/cli-reference.md) |
| `tools` | `rok-tools-cli`, linked | [rok-tools' README](https://github.com/ateeq1999/rok-tools#readme) |
| any other `<name>` | the executable `rok-<name>` on the `PATH`, with the remaining arguments | that executable's own |

```text
rok                      print this help (exit 0)
rok -h | --help | help   print this help (exit 0)
rok -V | --version       print `rok <version>` (exit 0)
rok docs [COMMAND…]      the documentation, served on 127.0.0.1 (see below)
rok <word> [ARGS…]       route as above
```

- rok-data's words are read from `rok_data_cli::command()` at run time, so
  a command rok-data adds is reachable as soon as `rok` links that
  release.
- `rok <name>` with no `rok-<name>` on the `PATH` is a usage error (exit
  2) naming the executable looked for. An argument starting with `-` in
  the first word's place, other than help and version, is a usage error
  too.
- A linked CLI's exit code, or a `rok-<name>`'s, is `rok`'s.
- `rok --help` lists, under "Installed", the `rok-*` executables found on
  the `PATH`, without the linked CLIs' own standalone binaries
  (`rok-data`, `rok-tools`), which only repeat a linked command.

## `rok docs [COMMAND…] [OPTIONS]`

`rok`'s own command: the documentation of every command it answers.

| Flag | Meaning |
| --- | --- |
| `COMMAND…` | Open this command's page (`rok docs db create`); none is the index |
| `-p, --port <PORT>` | Serve on this port, or the first free one of the next ten (default 4730; `0` lets the system pick) |
| `-n, --no-open` | Serve without opening the browser |
| `--print` | Write the page's Markdown on stdout instead of serving |

The pages are built from the command tree `rok` links: each command's
usage, arguments, options (with their defaults), subcommands, and the
examples its CLI ships (`rok_data_cli::examples`). They match the `rok`
running them. Guides (getting started, servers and databases, migrations,
generating code) are embedded with it.

It serves on `127.0.0.1` only, GET only, one request at a time, until
interrupted. The first line on stdout is the page's address, for scripts.
An unknown `COMMAND` is a usage error (exit 2).
