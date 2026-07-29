//! What is currently being dragged.
//!
//! Mullion has two drag sources that both land on the same drop targets (the
//! per-pane [`crate::components::drop_overlay::DropOverlay`]), but mean
//! different things on release:
//!
//! - dragging a pane's move handle or app icon **relocates an existing pane**
//! - dragging an activity out of the activity bar **creates a new pane** for it
//!
//! A single `Option<PaneId>` could only express the first, so the drag state is
//! a payload enum. Read it through [`crate::MullionContext::drag`], or via the
//! narrower [`crate::MullionContext::dragging_pane`] /
//! [`crate::MullionContext::dragging_activity`] helpers when you only care about
//! one kind.

use crate::tree::{ActivityId, PaneId};

/// The thing currently under the cursor in a drag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DragPayload {
    /// An existing pane is being relocated. Dropping it moves it; dropping it
    /// on itself is a no-op.
    Pane(PaneId),
    /// An activity was dragged out of an activity bar. Dropping it creates a
    /// new pane showing that activity.
    ///
    /// There is no pane id yet — the host mints one when the drop lands, via
    /// the `new_pane` hook ([`crate::PaneFactory`]).
    NewActivity(ActivityId),
}

impl DragPayload {
    /// The pane being relocated, if this is a pane drag.
    pub fn pane(&self) -> Option<&PaneId> {
        match self {
            DragPayload::Pane(id) => Some(id),
            DragPayload::NewActivity(_) => None,
        }
    }

    /// The activity being placed, if this is a new-activity drag.
    pub fn activity(&self) -> Option<&ActivityId> {
        match self {
            DragPayload::NewActivity(id) => Some(id),
            DragPayload::Pane(_) => None,
        }
    }

    /// Whether dropping on `pane` would be a no-op.
    ///
    /// Only a pane drag has a "self" to exclude; a new-activity drag is a valid
    /// drop on every pane, including the one whose bar it came from.
    pub fn is_self(&self, pane: &PaneId) -> bool {
        self.pane() == Some(pane)
    }
}
