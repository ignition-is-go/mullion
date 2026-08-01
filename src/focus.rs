use serde::{Deserialize, Serialize};

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
    Hover,
    /// Focus changes when the user presses the mouse inside a pane and remains
    /// there until another pane is clicked or focused programmatically.
    #[default]
    Click,
}
