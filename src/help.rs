//! `rok --help`: the linked CLIs' commands, as they describe themselves,
//! and the `rok-*` executables found on the `PATH`.

use std::fmt::Write as _;

use crate::{TOOLS, external, linked_words};

/// The root help text.
pub(crate) fn render() -> String {
    let tools = rok_tools_cli::command()
        .name(TOOLS)
        .about("Developer tools: grep, replace, loc, refactor, … (`rok tools list`)");
    let mut command = clap::Command::new("rok")
        .version(env!("CARGO_PKG_VERSION"))
        .about("One command for the rok toolchain")
        .disable_help_subcommand(true)
        .subcommands(rok_data_cli::command().get_subcommands().cloned())
        .subcommand(tools);
    let mut text = command.render_help().to_string();
    let installed = external::installed(&linked_words());
    text.push_str("\nInstalled (rok <name> runs rok-<name> from the PATH):\n");
    if installed.is_empty() {
        text.push_str("  none\n");
    }
    for name in installed {
        let _ = writeln!(text, "  {name}");
    }
    text.push_str(
        "\nEach command's own help: `rok <command> --help`. \
         Most commands accept --json for machine-readable output.\n",
    );
    text
}
