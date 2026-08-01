//! Optional `leptos-command-palette` integration.
//!
//! Enabled by Mullion's `command-palette` Cargo feature. The conversion
//! function lets applications merge Mullion's entries into their own command
//! registration flow; the component handles registration and cleanup for the
//! common case.

use leptos::prelude::*;
use leptos_command_palette::{use_command_palette, Command};

use crate::commands::{MullionCommands, PaneCommand};
use crate::tree::{PaneData, PaneId};

/// Build command-palette entries backed by a Mullion dispatcher.
pub fn mullion_palette_commands<D: PaneData + Send + Sync>(
    commands: MullionCommands<D>,
) -> Vec<Command> {
    let mut entries = Vec::new();

    let focus_commands = commands.clone();
    entries.push(
        Command::submenu("mullion.focus.pane", "Focus Pane…", move || {
            focus_commands
                .context()
                .pane_ids()
                .into_iter()
                .enumerate()
                .map(|(index, pane)| focus_pane_entry(focus_commands.clone(), pane, index))
                .collect()
        })
        .searchable_children()
        .description("Choose a pane from the live Mullion layout")
        .group("Mullion · Focus"),
    );

    for pane_command in PaneCommand::catalog() {
        if matches!(pane_command, PaneCommand::Split(_)) && !commands.can_split() {
            continue;
        }
        let action_commands = commands.clone();
        entries.push(
            Command::new(pane_command.id(), pane_command.name(), move || {
                let _ = action_commands.execute(pane_command);
            })
            .description(pane_command.description())
            .group(pane_command.group()),
        );
    }
    entries
}

fn focus_pane_entry<D: PaneData + Send + Sync>(
    commands: MullionCommands<D>,
    pane: PaneId,
    index: usize,
) -> Command {
    let id = format!("mullion.focus.pane.{}", pane.0);
    let name = format!("{} · {}", index + 1, pane.0);
    Command::new(id, name, move || {
        commands.context().focus_pane(&pane);
    })
    .description("Focus this pane")
    .group("Mullion · Focus")
}

/// Register Mullion's command catalog with the nearest command-palette
/// provider and unregister it when this component unmounts.
#[component]
pub fn MullionCommandPalette<D: PaneData + Send + Sync>(
    commands: MullionCommands<D>,
) -> impl IntoView {
    let palette = use_command_palette();
    let entries = mullion_palette_commands(commands);
    let ids: Vec<_> = entries.iter().map(|command| command.id.clone()).collect();
    palette.register_many(entries);

    on_cleanup(move || {
        for id in &ids {
            palette.unregister(id);
        }
    });
}
