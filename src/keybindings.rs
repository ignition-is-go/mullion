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
        let key = match normalize_key(&self.key).as_str() {
            "space" => "Space".into(),
            "arrowleft" => "←".into(),
            "arrowright" => "→".into(),
            "arrowup" => "↑".into(),
            "arrowdown" => "↓".into(),
            "escape" => "Esc".into(),
            other => other.to_uppercase(),
        };
        f.write_str(&key)
    }
}

fn normalize_key(key: &str) -> String {
    match key {
        " " | "Spacebar" => "space".into(),
        other => other.to_lowercase(),
    }
}

/// One command sequence after a keymap prefix.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MullionKeyBinding {
    pub sequence: Vec<KeyChord>,
    pub command: PaneCommand,
}

impl MullionKeyBinding {
    pub fn new(chord: KeyChord, command: PaneCommand) -> Self {
        Self {
            sequence: vec![chord],
            command,
        }
    }

    pub fn from_sequence(sequence: Vec<KeyChord>, command: PaneCommand) -> Self {
        Self { sequence, command }
    }
}

/// Prefix-based pane keymap.
///
/// Bindings may contain more than one chord after the prefix. This lets the
/// default map group operations mnemonically — for example, `Ctrl+M`, `M`,
/// `ArrowLeft` means "move the pane left" — without consuming common browser
/// or application shortcuts globally.
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

    /// Mullion's default `Ctrl+M` command map.
    ///
    /// Arrow keys navigate directly after the prefix. Multi-step groups use a
    /// mnemonic first key: `M`ove, `S`wap, `R`esize, `N`ew, `O`rient,
    /// `C`ycle, `L`ayout, and `F`ocus by number.
    pub fn mullion() -> Self {
        use PaneCommand::*;
        use PaneDirection::*;

        let mut map = Self::new(KeyChord::new("m").control());
        for (key, direction) in [
            ("ArrowLeft", Left),
            ("ArrowDown", Down),
            ("ArrowUp", Up),
            ("ArrowRight", Right),
        ] {
            map.bind(KeyChord::new(key), Focus(direction));
            map.bind_sequence([KeyChord::new("m"), KeyChord::new(key)], Move(direction));
            map.bind_sequence([KeyChord::new("s"), KeyChord::new(key)], Swap(direction));
            map.bind_sequence([KeyChord::new("r"), KeyChord::new(key)], Resize(direction));
        }

        map.bind(KeyChord::new("Tab"), FocusNext);
        map.bind(KeyChord::new("Tab").shift(), FocusPrevious);
        map.bind(KeyChord::new("Home"), FocusFirst);
        map.bind(KeyChord::new("End"), FocusLast);
        for index in 0..9 {
            map.bind_sequence(
                [KeyChord::new("f"), KeyChord::new((index + 1).to_string())],
                FocusIndex(index),
            );
        }

        // Splits always insert the new pane as the second child, which is the
        // pane to the right or below. Name the keys for that visible outcome
        // instead of exposing the ambiguous "horizontal/vertical split" terms.
        map.bind_sequence(
            [KeyChord::new("n"), KeyChord::new("r")],
            Split(SplitDirection::Horizontal),
        );
        map.bind_sequence(
            [KeyChord::new("n"), KeyChord::new("d")],
            Split(SplitDirection::Vertical),
        );
        map.bind(KeyChord::new("Delete"), Close);
        map.bind(KeyChord::new("Backspace"), Close);
        map.bind(KeyChord::new("Enter"), ToggleZoom);

        map.bind_sequence([KeyChord::new("s"), KeyChord::new("[")], SwapPrevious);
        map.bind_sequence([KeyChord::new("s"), KeyChord::new("]")], SwapNext);

        map.bind_sequence(
            [KeyChord::new("o"), KeyChord::new("r")],
            SetParentSplitDirection(SplitDirection::Horizontal),
        );
        map.bind_sequence(
            [KeyChord::new("o"), KeyChord::new("d")],
            SetParentSplitDirection(SplitDirection::Vertical),
        );
        map.bind_sequence(
            [KeyChord::new("o"), KeyChord::new("t")],
            ToggleParentSplitDirection,
        );
        map.bind(KeyChord::new("b"), Balance);

        map.bind_sequence(
            [KeyChord::new("c"), KeyChord::new("ArrowLeft")],
            Rotate(PaneRotation::Backward),
        );
        map.bind_sequence(
            [KeyChord::new("c"), KeyChord::new("ArrowRight")],
            Rotate(PaneRotation::Forward),
        );

        for (key, layout) in [
            ("1", PaneLayout::EvenHorizontal),
            ("2", PaneLayout::EvenVertical),
            ("3", PaneLayout::MainHorizontal),
            ("4", PaneLayout::MainVertical),
            ("5", PaneLayout::Tiled),
        ] {
            map.bind_sequence(
                [KeyChord::new("l"), KeyChord::new(key)],
                ApplyLayout(layout),
            );
        }
        map
    }

    /// An opt-in `Ctrl+B` map for applications whose users expect tmux.
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
        self.bind_sequence([chord], command);
    }

    /// Replace any existing binding for the exact post-prefix sequence.
    pub fn bind_sequence(
        &mut self,
        sequence: impl IntoIterator<Item = KeyChord>,
        command: PaneCommand,
    ) {
        let sequence: Vec<_> = sequence.into_iter().collect();
        if sequence.is_empty() {
            return;
        }
        self.bindings.retain(|binding| binding.sequence != sequence);
        self.bindings
            .push(MullionKeyBinding::from_sequence(sequence, command));
    }

    pub fn with_binding(mut self, chord: KeyChord, command: PaneCommand) -> Self {
        self.bind(chord, command);
        self
    }

    pub fn with_sequence(
        mut self,
        sequence: impl IntoIterator<Item = KeyChord>,
        command: PaneCommand,
    ) -> Self {
        self.bind_sequence(sequence, command);
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
            .map(|binding| {
                std::iter::once(&self.prefix)
                    .chain(binding.sequence.iter())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
    }

    fn match_sequence(&self, strokes: &[KeyStroke]) -> SequenceMatch {
        if let Some(binding) = self.bindings.iter().find(|binding| {
            binding.sequence.len() == strokes.len()
                && binding
                    .sequence
                    .iter()
                    .zip(strokes)
                    .all(|(chord, stroke)| chord.matches(stroke))
        }) {
            return SequenceMatch::Command(binding.command);
        }

        if self.bindings.iter().any(|binding| {
            binding.sequence.len() > strokes.len()
                && binding
                    .sequence
                    .iter()
                    .zip(strokes)
                    .all(|(chord, stroke)| chord.matches(stroke))
        }) {
            SequenceMatch::Pending
        } else {
            SequenceMatch::NoMatch
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SequenceMatch {
    Command(PaneCommand),
    Pending,
    NoMatch,
}

impl Default for MullionKeymap {
    fn default() -> Self {
        Self::mullion()
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
    let pending_strokes = RwSignal::new(Vec::<KeyStroke>::new());
    let handle = window_event_listener(leptos::ev::keydown, move |event| {
        if keymap.ignore_editable_targets && has_editable_target(&event) {
            prefix_active.set(false);
            pending_strokes.set(Vec::new());
            return;
        }

        let stroke = KeyStroke::from_event(&event);
        if keymap.prefix.matches(&stroke) {
            event.prevent_default();
            prefix_active.set(true);
            pending_strokes.set(Vec::new());
            return;
        }

        if !prefix_active.get_untracked() {
            return;
        }
        if normalize_key(&stroke.key) == "escape" {
            event.prevent_default();
            prefix_active.set(false);
            pending_strokes.set(Vec::new());
            return;
        }

        let mut strokes = pending_strokes.get_untracked();
        strokes.push(stroke);
        match keymap.match_sequence(&strokes) {
            SequenceMatch::Command(command) => {
                event.prevent_default();
                event.stop_propagation();
                prefix_active.set(false);
                pending_strokes.set(Vec::new());
                let _ = commands.execute(command);
            }
            SequenceMatch::Pending => {
                event.prevent_default();
                event.stop_propagation();
                pending_strokes.set(strokes);
            }
            SequenceMatch::NoMatch => {
                prefix_active.set(false);
                pending_strokes.set(Vec::new());
            }
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

    fn sequence(keys: &[&str]) -> Vec<KeyStroke> {
        keys.iter().map(|key| KeyStroke::new(*key)).collect()
    }

    #[test]
    fn mullion_default_groups_directional_actions_mnemonically() {
        let map = MullionKeymap::default();
        assert!(map.prefix().matches(&KeyStroke {
            key: "m".into(),
            control: true,
            alt: false,
            shift: false,
            meta: false,
        }));
        assert_eq!(
            map.match_sequence(&sequence(&["ArrowLeft"])),
            SequenceMatch::Command(PaneCommand::Focus(PaneDirection::Left))
        );
        assert_eq!(
            map.match_sequence(&sequence(&["m"])),
            SequenceMatch::Pending
        );
        assert_eq!(
            map.match_sequence(&sequence(&["m", "ArrowLeft"])),
            SequenceMatch::Command(PaneCommand::Move(PaneDirection::Left))
        );
        assert_eq!(
            map.match_sequence(&sequence(&["n", "r"])),
            SequenceMatch::Command(PaneCommand::Split(SplitDirection::Horizontal))
        );
    }

    #[test]
    fn mullion_default_binds_the_entire_static_command_catalog() {
        let map = MullionKeymap::default();
        for command in PaneCommand::catalog() {
            assert!(
                map.sequence_for(command).is_some(),
                "missing default key sequence for {command:?}"
            );
        }
        for index in 0..9 {
            assert!(map.sequence_for(PaneCommand::FocusIndex(index)).is_some());
        }
    }

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
            map.match_sequence(&sequence(&["h"])),
            SequenceMatch::Command(PaneCommand::Focus(PaneDirection::Left))
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
            map.match_sequence(&sequence(&["x"])),
            SequenceMatch::Command(PaneCommand::ToggleZoom)
        );
    }

    #[test]
    fn rebinding_replaces_only_the_exact_sequence() {
        let mut map = MullionKeymap::new(KeyChord::new("m").control());
        map.bind_sequence(
            [KeyChord::new("m"), KeyChord::new("ArrowLeft")],
            PaneCommand::Move(PaneDirection::Left),
        );
        map.bind_sequence(
            [KeyChord::new("m"), KeyChord::new("ArrowLeft")],
            PaneCommand::Swap(PaneDirection::Left),
        );
        map.bind(KeyChord::new("m"), PaneCommand::FocusNext);

        assert_eq!(map.bindings().len(), 2);
        assert_eq!(
            map.match_sequence(&sequence(&["m", "ArrowLeft"])),
            SequenceMatch::Command(PaneCommand::Swap(PaneDirection::Left))
        );
    }

    #[test]
    fn keymaps_round_trip_as_configuration() {
        let map = MullionKeymap::default();
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
