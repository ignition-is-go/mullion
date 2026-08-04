use serde::{Deserialize, Serialize};

/// Default focus accent shared by the separator frame and pane grabber.
///
/// Hosts can replace the whole value with `--ml-focus-color`. The fallback
/// keeps the primary hue but softens it against Mullion's pane border so the
/// focused pane reads clearly without becoming a neon box.
pub(crate) const FOCUS_COLOR: &str = "var(--ml-focus-color,color-mix(in srgb,var(--ml-primary,#00a4ef) 65%,var(--ml-border,#1a1a1a)))";

/// The pane grabber may be tuned separately while inheriting the focus accent
/// by default.
pub(crate) const FOCUSED_GRABBER_COLOR: &str = "var(--ml-focused-grabber-color,var(--ml-focus-color,color-mix(in srgb,var(--ml-primary,#00a4ef) 65%,var(--ml-border,#1a1a1a))))";

/// How pointer interaction changes the focused pane.
///
/// Programmatic focus commands work in either mode. This setting only controls
/// whether moving over a pane or clicking inside it acquires focus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaneFocusBehavior {
    /// Focus follows the pointer as it enters panes.
    ///
    /// This preserves Mullion's behavior before focus became a first-class
    /// interaction model.
    #[default]
    Hover,
    /// Focus changes when the user presses the mouse inside a pane and remains
    /// there until another pane is clicked or focused programmatically.
    Click,
}
