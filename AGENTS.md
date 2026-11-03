# AGENTS.md — working in rok-v2

`rok-v2` builds the `rok` binary (crate `rok-cli`), one member repo of
the rok toolchain (see `rok-workspace`, the folder around it; rok-data's
workspace ADR 0032 records the split). It owns routing and nothing else:
no command's grammar lives here.

## Where a change belongs

| Change | Where |
| --- | --- |
| A `rok db`, `rok migrate`, `rok gen`, `rok sqlc` … command or flag | rok-data (`crates/rok-cli`), then bump `rok-data-cli` here |
| A `rok tools …` Tool | rok-tools, then bump `rok-tools-cli` here |
| Which word goes where, the PATH fallback, `rok --help` | `src/lib.rs`, `src/external.rs`, `src/help.rs` |
| `rok docs`: a page's layout, a guide, the server | `src/docs/` (`pages.rs`, `guides/*.md`, `serve.rs`); a command's own text and examples belong to its CLI |
| `rok update`: the release lookup, the checksum, the archives, replacing the binary | `src/update/` (`release.rs`, `mod.rs`); `build.rs` records the target triple |
| Binary releases and the installers | `.github/workflows/release.yml`, `install.sh`, `install.ps1` |

## Rules

1. Never re-declare a linked CLI's grammar. rok-data's words are read
   from `rok_data_cli::command()`'s subcommands; tokens reach each CLI
   verbatim, named `rok` (`rok tools` for rok-tools) so their help reads
   `rok …`.
2. Exit codes: 0 for success and help, 2 for a usage error (an unknown
   word with no `rok-<word>` on the `PATH` included), 1 for a runtime
   failure; a linked CLI's or a `rok-<word>`'s own code passes through.
3. `cargo fmt --all` and `cargo clippy --all-targets -- -D warnings`
   clean; `tests/routing.rs` drives the built binary on every OS.
4. The linked CLIs come from crates.io. To build against unreleased
   member checkouts, use the meta-repo's patch file:
   `cargo build --config ../.cargo/patch-local.toml`.

## Releasing

Bump `version` in `Cargo.toml`, add a `## [X.Y.Z]` section to
`CHANGELOG.md`, merge, then tag **the merge commit** `vX.Y.Z` (check
`git show vX.Y.Z:Cargo.toml` first). `release.yml` builds the binaries,
publishes the GitHub release and `rok-cli`, and verifies both
installers. A run stopped by a runner is finished by running the
workflow by hand with the same tag.
