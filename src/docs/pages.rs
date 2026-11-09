//! The documentation as Markdown: an index, one page per command (built
//! from the command tree `rok` links, so it matches this binary), and the
//! guides embedded with it.

use std::fmt::Write as _;

use clap::{Arg, Command};

use crate::{external, help, linked_words};

/// A guide: its address, title and Markdown.
pub(crate) struct Guide {
    pub(crate) slug: &'static str,
    pub(crate) title: &'static str,
    pub(crate) markdown: &'static str,
}

/// The guides `rok docs` ships, in reading order.
pub(crate) const GUIDES: &[Guide] = &[
    Guide {
        slug: "getting-started",
        title: "Getting started",
        markdown: include_str!("guides/getting-started.md"),
    },
    Guide {
        slug: "servers-and-databases",
        title: "Servers and databases",
        markdown: include_str!("guides/servers-and-databases.md"),
    },
    Guide {
        slug: "migrations",
        title: "Migrations",
        markdown: include_str!("guides/migrations.md"),
    },
    Guide {
        slug: "generating-code",
        title: "Generating code",
        markdown: include_str!("guides/generating-code.md"),
    },
];

/// The page address of the command at `path` (`["db", "create"]`).
pub(crate) fn command_href(path: &[&str]) -> String {
    format!("/commands/{}", path.join("-"))
}

/// The command tree, built (so usage lines and bin names are filled in).
fn built_tree() -> Command {
    let mut tree = help::tree();
    tree.build();
    tree
}

/// The command at `path` in `tree`, if there is one.
fn find<'a>(tree: &'a Command, path: &[&str]) -> Option<&'a Command> {
    path.iter()
        .try_fold(tree, |command, name| command.find_subcommand(name))
}

/// The page of the command at `path`, or `None` when no such command.
pub(crate) fn command_page(path: &[&str]) -> Option<String> {
    let tree = built_tree();
    let command = find(&tree, path)?;
    if path.is_empty() {
        return None;
    }
    Some(render_command(command, path))
}

/// The command a page address (`db-create`) names, as a path.
pub(crate) fn path_of(slug: &str) -> Option<Vec<String>> {
    let tree = built_tree();
    let mut path = Vec::new();
    let mut command = &tree;
    let mut rest = slug;
    // Command names may hold `-` (`check-arch`), so take the longest
    // subcommand name that starts the remaining slug at each step.
    while !rest.is_empty() {
        let (sub, after) = command
            .get_subcommands()
            .filter_map(|sub| {
                let name = sub.get_name();
                let after = rest.strip_prefix(name)?;
                (after.is_empty() || after.starts_with('-'))
                    .then(|| (sub, after.trim_start_matches('-')))
            })
            .max_by_key(|(sub, _)| sub.get_name().len())?;
        path.push(sub.get_name().to_owned());
        command = sub;
        rest = after;
    }
    Some(path)
}

fn render_command(command: &Command, path: &[&str]) -> String {
    let mut page = format!("# rok {}\n\n", path.join(" "));
    if let Some(about) = command.get_long_about().or(command.get_about()) {
        let _ = writeln!(page, "{about}\n");
    }

    let usage = command.clone().render_usage().to_string();
    let usage = usage.trim_start_matches("Usage: ").trim();
    let _ = writeln!(page, "## Usage\n\n```text\n{usage}\n```\n");

    let (positional, options): (Vec<&Arg>, Vec<&Arg>) = command
        .get_arguments()
        .filter(|arg| !arg.is_hide_set() && !matches!(arg.get_id().as_str(), "help" | "version"))
        .partition(|arg| arg.is_positional());
    if !positional.is_empty() {
        page.push_str("## Arguments\n\n| Argument | Description |\n| --- | --- |\n");
        for arg in positional {
            let name = arg
                .get_value_names()
                .and_then(|names| names.first())
                .map_or_else(
                    || arg.get_id().to_string().to_uppercase(),
                    ToString::to_string,
                );
            let _ = writeln!(page, "| `<{name}>` | {} |", describe(arg));
        }
        page.push('\n');
    }
    if !options.is_empty() {
        page.push_str("## Options\n\n| Option | Description |\n| --- | --- |\n");
        for arg in options {
            let _ = writeln!(page, "| `{}` | {} |", flag(arg), describe(arg));
        }
        page.push('\n');
    }

    let subcommands = listed(command);
    if !subcommands.is_empty() {
        page.push_str("## Commands\n\n");
        for sub in subcommands {
            let mut sub_path: Vec<&str> = path.to_vec();
            sub_path.push(sub.get_name());
            let about = sub.get_about().map(ToString::to_string).unwrap_or_default();
            let _ = writeln!(
                page,
                "- [`rok {}`]({}) — {about}",
                sub_path.join(" "),
                command_href(&sub_path)
            );
        }
        page.push('\n');
    }

    if let Some(examples) = rok_data_cli::examples::examples(&path.join(" ")) {
        page.push_str("## Examples\n\n");
        for example in examples {
            let _ = writeln!(
                page,
                "{}\n\n```sh\n{}\n```\n",
                example.what, example.command
            );
        }
    }
    page
}

/// `command`'s visible subcommands in the order its help lists them: by
/// display order (the order they were declared), not the order they are
/// stored in, which editing a subcommand changes.
fn listed(command: &Command) -> Vec<&Command> {
    let mut subcommands: Vec<&Command> = command
        .get_subcommands()
        .filter(|sub| !sub.is_hide_set())
        .collect();
    subcommands.sort_by_key(|sub| sub.get_display_order());
    subcommands
}

/// `-s, --server <NAME>` for an option.
fn flag(arg: &Arg) -> String {
    let mut text = String::new();
    if let Some(short) = arg.get_short() {
        let _ = write!(text, "-{short}");
        if arg.get_long().is_some() {
            text.push_str(", ");
        }
    }
    if let Some(long) = arg.get_long() {
        let _ = write!(text, "--{long}");
    }
    if arg.get_action().takes_values()
        && let Some(names) = arg.get_value_names()
    {
        for name in names {
            let _ = write!(text, " <{name}>");
        }
    }
    text
}

/// An argument's help, on one table row, with its default if it has one.
fn describe(arg: &Arg) -> String {
    let help = arg
        .get_long_help()
        .or(arg.get_help())
        .map(ToString::to_string)
        .unwrap_or_default();
    let mut text = help.replace('\n', " ").replace('|', "\\|");
    // A flag that takes no value (`--json`) "defaults" to off; saying so is noise.
    let defaults: Vec<String> = if arg.get_action().takes_values() {
        arg.get_default_values()
            .iter()
            .map(|value| value.to_string_lossy().into_owned())
            .collect()
    } else {
        Vec::new()
    };
    if !defaults.is_empty() && !text.contains("default") {
        let _ = write!(text, " (default: `{}`)", defaults.join(", "));
    }
    text
}

/// The front page: the guides, every command, and what the `PATH` adds.
pub(crate) fn index() -> String {
    let tree = built_tree();
    let mut page = format!(
        "# rok {}\n\nOne command for the rok toolchain. Each page here comes from the \
         `rok` you are running, so it matches its version.\n\n## Guides\n\n",
        env!("CARGO_PKG_VERSION")
    );
    for guide in GUIDES {
        let _ = writeln!(page, "- [{}](/guides/{})", guide.title, guide.slug);
    }
    page.push_str("\n## Commands\n\n");
    for command in listed(&tree) {
        let about = command
            .get_about()
            .map(ToString::to_string)
            .unwrap_or_default();
        let _ = writeln!(
            page,
            "- [`rok {}`]({}) — {about}",
            command.get_name(),
            command_href(&[command.get_name()])
        );
    }
    let installed = external::installed(&linked_words());
    if !installed.is_empty() {
        page.push_str(
            "\n## Installed\n\n`rok <name>` runs these `rok-<name>` executables from the PATH; \
             their own `--help` documents them.\n\n",
        );
        for name in installed {
            let _ = writeln!(page, "- `rok {name}`");
        }
    }
    page
}

/// The guide at `slug`, if there is one.
pub(crate) fn guide(slug: &str) -> Option<&'static Guide> {
    GUIDES.iter().find(|guide| guide.slug == slug)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_command_page_has_usage_options_and_examples() {
        let page = command_page(&["db", "create"]).expect("rok db create");
        assert!(page.starts_with("# rok db create\n"), "{page}");
        assert!(
            page.contains("## Usage\n\n```text\nrok db create"),
            "{page}"
        );
        assert!(page.contains("| `--server <NAME>` |"), "{page}");
        assert!(page.contains("| `<NAME>` |"), "{page}");
        assert!(page.contains("## Examples"), "{page}");
        assert!(
            page.contains("rok db create shop_dev --server local"),
            "{page}"
        );
    }

    #[test]
    fn a_group_page_links_its_commands() {
        let page = command_page(&["migrate"]).expect("rok migrate");
        assert!(
            page.contains("- [`rok migrate up`](/commands/migrate-up)"),
            "{page}"
        );
    }

    #[test]
    fn an_unknown_command_has_no_page() {
        assert!(command_page(&["nope"]).is_none());
        assert!(command_page(&[]).is_none());
    }

    #[test]
    fn page_addresses_round_trip_even_with_dashed_names() {
        assert_eq!(
            path_of("db-create"),
            Some(vec!["db".into(), "create".into()])
        );
        assert_eq!(
            path_of("sqlc-check-arch"),
            Some(vec!["sqlc".into(), "check-arch".into()])
        );
        assert_eq!(path_of("nope"), None);
    }

    #[test]
    fn the_index_lists_every_guide_and_command() {
        let page = index();
        for guide in GUIDES {
            assert!(
                page.contains(&format!("(/guides/{})", guide.slug)),
                "{page}"
            );
        }
        for command in [
            "init", "gen", "migrate", "db", "server", "tools", "dev", "port", "docs",
        ] {
            assert!(
                page.contains(&format!("(/commands/{command})")),
                "{command}: {page}"
            );
        }
    }
}
