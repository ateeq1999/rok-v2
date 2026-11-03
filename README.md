# rok

One command for the rok toolchain. `rok` links the toolchain's command
lines and runs any other `rok <name>` as `rok-<name>` from your `PATH`,
the way `cargo` and `git` find their subcommands.

| `rok …` | answered by |
| --- | --- |
| `init`, `gen`, `generate`, `migrate`, `db`, `server`, `schema`, `sqlc` | [rok-data](https://github.com/ateeq1999/rok-data)'s CLI (`rok-data-cli`), linked in |
| `tools …` | [rok-tools](https://github.com/ateeq1999/rok-tools)' CLI (`rok-tools-cli`), linked in |
| anything else, `rok <name> …` | the executable `rok-<name>` on your `PATH` |

`rok docs` opens every command's documentation in your browser, served on
localhost: a page per command (its usage, options and examples, from the
`rok` you run) and short guides. `rok docs db create` opens that command's
page; `--print` writes the Markdown instead.

`rok --help` lists the linked commands and, under "Installed", every
`rok-*` executable it finds. Each command's own help is
`rok <command> --help`.

## Install

```sh
cargo install rok-cli         # provides the `rok` binary
```

Or take a prebuilt binary: the GitHub releases carry checksummed archives
for Linux (musl), macOS and Windows, x86_64 and ARM64, and the installers
download, verify and install them:

```sh
curl -fsSL https://raw.githubusercontent.com/ateeq1999/rok-v2/main/install.sh | sh
```

```powershell
irm https://raw.githubusercontent.com/ateeq1999/rok-v2/main/install.ps1 | iex
```

`rok-cli` up to 0.6.1 was an older task runner, now archived; from 0.7.0
the crate is this command.

## Adding a command

A command of its own needs no change here: install an executable named
`rok-<name>` on the `PATH` and `rok <name> …` runs it with the remaining
arguments and returns its exit code. Its `--json` and exit codes should
follow the toolchain's contract: 0 for success and help, 2 for a usage
error, 1 for a runtime failure; `--json` prints only machine lines on
stdout.

## License

MIT OR Apache-2.0.
