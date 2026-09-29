//! Resolve public command paths and aliases from the executable's clap tree.

use clap::{Command, CommandFactory};

use crate::cli::Cli;

pub(super) fn resolve(path: &[String]) -> Option<(Command, Vec<String>)> {
    let mut root = Cli::command();
    root.build();
    let mut command = &root;
    let mut canonical = Vec::new();
    for segment in path {
        command = command.get_subcommands().find(|child| {
            !child.is_hide_set()
                && child.get_name() != "help"
                && (child.get_name() == segment
                    || child.get_all_aliases().any(|alias| alias == segment))
        })?;
        canonical.push(command.get_name().to_owned());
    }
    Some((command.clone(), canonical))
}
