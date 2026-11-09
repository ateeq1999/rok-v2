//! `rok --help`: the linked CLIs' commands, as they describe themselves,
//! `rok`'s own, and the `rok-*` executables found on the `PATH`.

use std::fmt::Write as _;

use crate::{TOOLS, docs, external, linked_words, update};

/// Every command `rok` answers, as one tree: rok-data's, `tools`, rok-dev's
/// (`dev`, `run`, `doctor`, `port`), and `rok`'s own (`docs`, `update`).
/// `rok --help` and `rok docs` both read it.
pub(crate) fn tree() -> clap::Command {
    let tools = rok_tools_cli::command()
        .name(TOOLS)
        .about("Developer tools: grep, replace, loc, refactor, … (`rok tools list`)");
    let commands = in_order(&rok_data_cli::command())
        .into_iter()
        .chain([tools])
        .chain(in_order(&rok_dev_cli::command()))
        .chain([docs::command(), update::command()])
        // A linked CLI's commands keep their own display order, which
        // would interleave the CLIs; number them in this order instead.
        .enumerate()
        .map(|(order, command)| command.display_order(order));
    clap::Command::new("rok")
        .version(env!("CARGO_PKG_VERSION"))
        .about("One command for the rok toolchain")
        .disable_help_subcommand(true)
        .subcommands(commands)
}

/// A linked CLI's commands, in the order its own help lists them.
fn in_order(cli: &clap::Command) -> Vec<clap::Command> {
    let mut commands: Vec<clap::Command> = cli.get_subcommands().cloned().collect();
    commands.sort_by_key(|command| (command.get_display_order(), command.get_name().to_owned()));
    commands
}

/// The root help text.
pub(crate) fn render() -> String {
    let mut text = tree().render_help().to_string();
    let installed = external::installed(&linked_words());
    text.push_str("\nInstalled (rok <name> runs rok-<name> from the PATH):\n");
    if installed.is_empty() {
        text.push_str("  none\n");
    }
    for name in installed {
        let _ = writeln!(text, "  {name}");
    }
    text.push_str(
        "\nEach command's own help: `rok <command> --help`, or `rok docs <command>` in the browser. \
         Most commands accept --json for machine-readable output.\n",
    );
    text
}
