use std::fmt;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;

use crate::commands::{MullionCommands, PaneCommand};
use crate::tree::{PaneData, PaneDirection, PaneLayout, PaneRotation, SplitDirection};

/// A platform-neutral keyboard event snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyStroke {
    pub key: String,
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl KeyStroke {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            control: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    fn from_event(event: &web_sys::KeyboardEvent) -> Self {
        Self {
            key: event.key(),
            control: event.ctrl_key(),
            alt: event.alt_key(),
            shift: event.shift_key(),
            meta: event.meta_key(),
        }
    }
}

/// A key plus its exact modifier set.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyChord {
    pub key: String,
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl KeyChord {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            control: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    pub fn control(mut self) -> Self {
        self.control = true;
        self
    }

    pub fn alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn meta(mut self) -> Self {
        self.meta = true;
        self
    }

    pub fn matches(&self, stroke: &KeyStroke) -> bool {
        normalize_key(&self.key) == normalize_key(&stroke.key)
            && self.control == stroke.control
            && self.alt == stroke.alt
            && self.shift == stroke.shift
            && self.meta == stroke.meta
    }
}

impl fmt::Display for KeyChord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.control {
            f.write_str("Ctrl+")?;
        }
        if self.alt {
            f.write_str("Alt+")?;
        }
        if self.shift {
            f.write_str("Shift+")?;
        }
        if self.meta {
            f.write_str("Meta+")?;
        }
        if normalize_key(&self.key) == "space" {
            f.write_str("Space")
        } else {
            f.write_str(&self.key.to_uppercase())
        }
    }
}

fn normalize_key(key: &str) -> String {
    match key {
        " " | "Spacebar" => "space".into(),
        other => other.to_lowercase(),
    }
}

/// One second-stage binding in a prefix keymap.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MullionKeyBinding {
    pub chord: KeyChord,
    pub command: PaneCommand,
}

impl MullionKeyBinding {
    pub fn new(chord: KeyChord, command: PaneCommand) -> Self {
        Self { chord, command }
    }
}

/// Prefix-based pane keymap, modeled after terminal multiplexers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MullionKeymap {
    prefix: KeyChord,
    bindings: Vec<MullionKeyBinding>,
    ignore_editable_targets: bool,
}

impl MullionKeymap {
    pub fn new(prefix: KeyChord) -> Self {
        Self {
            prefix,
            bindings: Vec::new(),
            ignore_editable_targets: true,
        }
    }

    /// The default `Ctrl+B` tmux-style command map.
    pub fn tmux() -> Self {
        use PaneCommand::*;
        use PaneDirection::*;

        let mut map = Self::new(KeyChord::new("b").control());
        for (key, direction) in [
            ("h", Left),
            ("j", Down),
            ("k", Up),
            ("l", Right),
            ("ArrowLeft", Left),
            ("ArrowDown", Down),
            ("ArrowUp", Up),
            ("ArrowRight", Right),
        ] {
            map.bind(KeyChord::new(key), Focus(direction));
        }

        map.bind(KeyChord::new("o"), FocusNext);
        map.bind(KeyChord::new(";"), FocusPrevious);
        for index in 0..9 {
            map.bind(KeyChord::new((index + 1).to_string()), FocusIndex(index));
        }

        // tmux's `%` creates a left/right split and `"` a top/bottom split.
        map.bind(
            KeyChord::new("%").shift(),
            Split(SplitDirection::Horizontal),
        );
        map.bind(KeyChord::new("\"").shift(), Split(SplitDirection::Vertical));
        map.bind(KeyChord::new("x"), Close);
        map.bind(KeyChord::new("z"), ToggleZoom);
        map.bind(KeyChord::new("Space"), ToggleParentSplitDirection);
        map.bind(KeyChord::new("e"), Balance);

        map.bind(KeyChord::new("{").shift(), SwapPrevious);
        map.bind(KeyChord::new("}").shift(), SwapNext);
        map.bind(KeyChord::new("o").control(), Rotate(PaneRotation::Forward));
        map.bind(KeyChord::new("o").alt(), Rotate(PaneRotation::Backward));

        for (key, direction) in [
            ("ArrowLeft", Left),
            ("ArrowDown", Down),
            ("ArrowUp", Up),
            ("ArrowRight", Right),
        ] {
            map.bind(KeyChord::new(key).shift(), Move(direction));
            map.bind(KeyChord::new(key).alt(), Swap(direction));
            map.bind(KeyChord::new(key).control(), Resize(direction));
        }

        for (key, direction) in [("h", Left), ("j", Down), ("k", Up), ("l", Right)] {
            map.bind(KeyChord::new(key).shift(), Move(direction));
            map.bind(KeyChord::new(key).alt(), Swap(direction));
            map.bind(KeyChord::new(key).control(), Resize(direction));
        }

        for (key, layout) in [
            ("1", PaneLayout::EvenHorizontal),
            ("2", PaneLayout::EvenVertical),
            ("3", PaneLayout::MainHorizontal),
            ("4", PaneLayout::MainVertical),
            ("5", PaneLayout::Tiled),
        ] {
            map.bind(KeyChord::new(key).alt(), ApplyLayout(layout));
        }
        map
    }

    pub fn prefix(&self) -> &KeyChord {
        &self.prefix
    }

    pub fn bindings(&self) -> &[MullionKeyBinding] {
        &self.bindings
    }

    /// Replace any existing binding for `chord`.
    pub fn bind(&mut self, chord: KeyChord, command: PaneCommand) {
        self.bindings.retain(|binding| binding.chord != chord);
        self.bindings.push(MullionKeyBinding::new(chord, command));
    }

    pub fn with_binding(mut self, chord: KeyChord, command: PaneCommand) -> Self {
        self.bind(chord, command);
        self
    }

    /// Configure whether sequences originating in inputs, textareas, selects,
    /// or content-editable elements may be captured. Defaults to `false`.
    pub fn capture_editable_targets(mut self, capture: bool) -> Self {
        self.ignore_editable_targets = !capture;
        self
    }

    /// Display the full prefix sequence for a command, if it is bound.
    pub fn sequence_for(&self, command: PaneCommand) -> Option<String> {
        self.bindings
            .iter()
            .find(|binding| binding.command == command)
            .map(|binding| format!("{}, {}", self.prefix, binding.chord))
    }

    fn command_for(&self, stroke: &KeyStroke) -> Option<PaneCommand> {
        self.bindings
            .iter()
            .find(|binding| binding.chord.matches(stroke))
            .map(|binding| binding.command)
    }
}

impl Default for MullionKeymap {
    fn default() -> Self {
        Self::tmux()
    }
}

/// Mount the global listener for a prefix-based Mullion keymap.
///
/// This component renders no DOM and should be mounted once. It is opt-in so a
/// library upgrade never starts consuming an application's existing shortcuts.
#[component]
pub fn MullionKeybindings<D: PaneData + Send + Sync>(
    commands: MullionCommands<D>,
    #[prop(optional)] keymap: MullionKeymap,
) -> impl IntoView {
    let prefix_active = RwSignal::new(false);
    let handle = window_event_listener(leptos::ev::keydown, move |event| {
        if keymap.ignore_editable_targets && has_editable_target(&event) {
            prefix_active.set(false);
            return;
        }

        let stroke = KeyStroke::from_event(&event);
        if keymap.prefix.matches(&stroke) {
            event.prevent_default();
            prefix_active.set(true);
            return;
        }

        if !prefix_active.get_untracked() {
            return;
        }
        prefix_active.set(false);
        if normalize_key(&stroke.key) == "escape" {
            event.prevent_default();
            return;
        }
        if let Some(command) = keymap.command_for(&stroke) {
            event.prevent_default();
            event.stop_propagation();
            let _ = commands.execute(command);
        }
    });

    on_cleanup(move || handle.remove());
}

fn has_editable_target(event: &web_sys::KeyboardEvent) -> bool {
    let Some(target) = event.target() else {
        return false;
    };
    let Ok(element) = target.dyn_into::<web_sys::Element>() else {
        return false;
    };
    if matches!(element.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT") {
        return true;
    }
    element
        .dyn_ref::<web_sys::HtmlElement>()
        .is_some_and(web_sys::HtmlElement::is_content_editable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tmux_prefix_and_navigation_are_bound() {
        let map = MullionKeymap::tmux();
        assert!(map.prefix().matches(&KeyStroke {
            key: "b".into(),
            control: true,
            alt: false,
            shift: false,
            meta: false,
        }));
        assert_eq!(
            map.command_for(&KeyStroke::new("h")),
            Some(PaneCommand::Focus(PaneDirection::Left))
        );
    }

    #[test]
    fn chords_require_an_exact_modifier_set() {
        let chord = KeyChord::new("ArrowLeft").control();
        let mut stroke = KeyStroke::new("ArrowLeft");
        assert!(!chord.matches(&stroke));
        stroke.control = true;
        assert!(chord.matches(&stroke));
        stroke.shift = true;
        assert!(!chord.matches(&stroke));
    }

    #[test]
    fn space_spellings_match() {
        assert!(KeyChord::new("Space").matches(&KeyStroke::new(" ")));
    }

    #[test]
    fn rebinding_replaces_the_existing_chord() {
        let mut map = MullionKeymap::new(KeyChord::new("b").control());
        map.bind(KeyChord::new("x"), PaneCommand::Close);
        map.bind(KeyChord::new("x"), PaneCommand::ToggleZoom);
        assert_eq!(map.bindings().len(), 1);
        assert_eq!(
            map.command_for(&KeyStroke::new("x")),
            Some(PaneCommand::ToggleZoom)
        );
    }

    #[test]
    fn keymaps_round_trip_as_configuration() {
        let map = MullionKeymap::tmux();
        let json = serde_json::to_string(&map).unwrap();
        let restored: MullionKeymap = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.prefix(), map.prefix());
        assert_eq!(restored.bindings(), map.bindings());
        assert_eq!(
            restored.ignore_editable_targets,
            map.ignore_editable_targets
        );
    }
}
