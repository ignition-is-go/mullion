use crate::tree::{ActivityId, DropEdge, PaneData, PaneId, PaneNode, SplitDirection};

/// Events emitted by the pane system. Subscribe to these for persistence.
#[derive(Clone, Debug)]
pub enum PaneEvent<D: PaneData> {
    /// A pane was split. Contains the target pane, direction, and the new pane's id/data.
    Split {
        target: PaneId,
        direction: SplitDirection,
        new_id: PaneId,
        new_data: D,
    },
    /// A pane was closed.
    Closed { id: PaneId, data: D },
    /// A split was resized. `split_key` is the first leaf id under the
    /// split's `second` subtree — the same key used to address splits
    /// throughout the API.
    Resized { split_key: PaneId, ratio: f64 },
    /// A pane was moved to a new position.
    Moved {
        source: PaneId,
        destination: PaneId,
        edge: DropEdge,
    },
    /// An activity was dragged out of an activity bar and dropped into the
    /// layout, creating a new pane for it.
    ///
    /// `new_id` and `new_data` are whatever the host's `new_pane` hook
    /// ([`crate::PaneFactory`]) returned, so the host can persist the pane it
    /// just minted. The tree has already been mutated when this fires; a
    /// `TreeChanged` follows.
    ActivityDropped {
        activity: ActivityId,
        destination: PaneId,
        edge: DropEdge,
        new_id: PaneId,
        new_data: D,
    },
    /// The split direction of a pane's parent was changed.
    DirectionChanged {
        pane: PaneId,
        direction: SplitDirection,
    },
    /// The active activity in a pane changed.
    ActivityChanged {
        pane: PaneId,
        activity: Option<ActivityId>,
    },
    /// A full tree snapshot (emitted after every mutation for convenience).
    TreeChanged { tree: PaneNode<D> },
}
