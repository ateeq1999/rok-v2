# Changelog

All notable changes to `rok` (the `rok-cli` crate) are documented here.
The format is [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and the version numbers follow [Semantic
Versioning](https://semver.org/spec/v2.0.0.html). Release notes for a tag
are this file's `## [X.Y.Z]` section.

## [Unreleased]

## [0.8.0] — `rok docs`, `rok update`, and rok-data 0.8

**Added**

- `rok update`: install the latest release (or `--version`'s) over the running
  `rok`. `--check` only reports, `--json` reports as one object. A `rok` from
  `cargo install` updates through Cargo; any other downloads its archive,
  verifies its SHA-256 (a mismatch installs nothing) and replaces itself.
- `rok docs`: the documentation of every command, served on localhost and
  opened in the browser (`rok docs db create` for one command's page;
  `--port`, `--no-open`; `--print` for the Markdown). Command pages come
  from the command tree `rok` links, with each command's examples, so they
  match the running version; four guides are embedded.

**Changed**

- `rok` links `rok-data-cli` 0.8.0 (0.7.0 linked 0.6.1): `rok server update`, an
  Examples section in every command's `--help`, and `framework = "grpc"`
  for `rok gen` (a `.proto` per Resource and a tonic service per feature).

## [0.7.0] — one command for the rok toolchain

The first release from this repository (rok-data's workspace ADR 0032).
The `rok-cli` crate up to 0.6.1 was an older task runner, now archived.

**Added**

- `rok` routes its first word: rok-data's commands (`init`, `gen`,
  `generate`, `migrate`, `db`, `server`, `schema`, `sqlc`) to
  `rok-data-cli`, `tools` to `rok-tools-cli`, both linked in, and any
  other word `x` to the executable `rok-x` on the `PATH`, whose exit code
  it returns.
- `rok --help` lists the linked commands as their CLIs describe them,
  and the `rok-*` executables it finds on the `PATH`.
- Release archives for Linux (musl), macOS and Windows, x86_64 and
  ARM64, with `install.sh` and `install.ps1`.
