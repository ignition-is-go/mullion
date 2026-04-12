use leptos::prelude::*;

use crate::activity::{ActivityDef, Category};
use crate::events::PaneEvent;
use crate::theme::{ActivityBarTheme, MullionTheme, PaneTheme, SplitHandleTheme};
use crate::tree::{ActivityId, DropEdge, PaneData, PaneId, PaneNode, SplitDirection};

/// The reactive store for the mullion pane system.
///
/// Provided via Leptos context at `<MullionRoot>`. The consuming app interacts
/// with panes through this context.
#[derive(Clone)]
pub struct MullionContext<D: PaneData> {
    /// The reactive pane tree. Can be read to render, updated for mutations.
    pub tree: RwSignal<PaneNode<D>>,
    /// All registered activity definitions.
    pub activities: StoredValue<Vec<ActivityDef<D>>>,
    /// Ordered categories.
    pub categories: StoredValue<Vec<Category>>,
    /// Event sink — write end. Every mutation pushes an event here.
    event_tx: StoredValue<Box<dyn Fn(PaneEvent<D>) + Send + Sync>>,
    /// Counter for generating unique PaneIds.
    next_id: RwSignal<u64>,
    /// The pane the mouse is currently over.
    pub focused_pane: RwSignal<Option<PaneId>>,
    /// Resolved themes (captured at provider time so they work in reactive closures).
    pub mullion_theme: MullionTheme,
    pub activity_bar_theme: ActivityBarTheme,
    pub split_handle_theme: SplitHandleTheme,
    pub pane_theme: PaneTheme,
}

impl<D: PaneData + Send + Sync> MullionContext<D> {
    pub fn new(
        initial_tree: PaneNode<D>,
        activities: Vec<ActivityDef<D>>,
        categories: Vec<Category>,
        event_handler: impl Fn(PaneEvent<D>) + Send + Sync + 'static,
        mullion_theme: MullionTheme,
        activity_bar_theme: ActivityBarTheme,
        split_handle_theme: SplitHandleTheme,
        pane_theme: PaneTheme,
    ) -> Self {
        let max_id = initial_tree
            .leaf_ids()
            .into_iter()
            .map(|id| id.0)
            .max()
            .unwrap_or(0);

        MullionContext {
            tree: RwSignal::new(initial_tree),
            activities: StoredValue::new(activities),
            categories: StoredValue::new(categories),
            event_tx: StoredValue::new(Box::new(event_handler)),
            next_id: RwSignal::new(max_id + 1),
            focused_pane: RwSignal::new(None),
            mullion_theme,
            activity_bar_theme,
            split_handle_theme,
            pane_theme,
        }
    }

    fn emit(&self, event: PaneEvent<D>) {
        self.event_tx.with_value(|tx| tx(event));
    }

    fn emit_tree_changed(&self) {
        let tree = self.tree.get_untracked();
        self.emit(PaneEvent::TreeChanged { tree });
    }

    fn alloc_id(&self) -> PaneId {
        let id = self.next_id.get_untracked();
        self.next_id.set(id + 1);
        PaneId(id)
    }

    /// Split a pane. Returns the new pane's id.
    pub fn split_pane(
        &self,
        target: PaneId,
        direction: SplitDirection,
        new_data: D,
    ) -> PaneId {
        let new_id = self.alloc_id();
        self.tree.update(|tree| {
            tree.split(target, direction, new_id, new_data.clone());
        });
        self.emit(PaneEvent::Split {
            target,
            direction,
            new_id,
            new_data,
        });
        self.emit_tree_changed();
        new_id
    }

    /// Close a pane. Returns the closed pane's data if found.
    pub fn close_pane(&self, id: PaneId) -> Option<D> {
        let mut closed_data = None;
        self.tree.update(|tree| {
            closed_data = tree.close(id);
        });
        if let Some(ref data) = closed_data {
            self.emit(PaneEvent::Closed {
                id,
                data: data.clone(),
            });
            self.emit_tree_changed();
        }
        closed_data
    }

    /// Resize the split containing a pane.
    pub fn resize_pane(&self, pane: PaneId, ratio: f64) {
        self.tree.update(|tree| {
            tree.set_ratio(pane, ratio);
        });
        self.emit(PaneEvent::Resized { pane, ratio });
        self.emit_tree_changed();
    }

    /// Change the split direction of a pane's parent.
    pub fn change_split_direction(&self, pane: PaneId, direction: SplitDirection) {
        self.tree.update(|tree| {
            tree.change_direction(pane, direction);
        });
        self.emit(PaneEvent::DirectionChanged { pane, direction });
        self.emit_tree_changed();
    }

    /// Move a pane to a new position relative to a destination pane.
    pub fn move_pane(&self, source: PaneId, destination: PaneId, edge: DropEdge) {
        let mut success = false;
        self.tree.update(|tree| {
            success = tree.move_pane(source, destination, edge);
        });
        if success {
            self.emit(PaneEvent::Moved {
                source,
                destination,
                edge,
            });
            self.emit_tree_changed();
        }
    }

    /// Set the active activity for a pane.
    pub fn set_active_activity(&self, pane: PaneId, activity: Option<ActivityId>) {
        self.tree.update(|tree| {
            if let Some(PaneNode::Leaf {
                active_activity, ..
            }) = tree.find_mut(pane)
            {
                *active_activity = activity;
            }
        });
        self.emit(PaneEvent::ActivityChanged { pane, activity });
        self.emit_tree_changed();
    }

    /// Get activities available in a pane, filtered by its data.
    pub fn activities_for_pane(&self, data: &D) -> Vec<ActivityDef<D>> {
        self.activities.with_value(|acts| {
            acts.iter()
                .filter(|a| (a.filter)(data))
                .cloned()
                .collect()
        })
    }

    /// Get categories sorted by order.
    pub fn sorted_categories(&self) -> Vec<Category> {
        self.categories.with_value(|cats| {
            let mut sorted = cats.clone();
            sorted.sort_by_key(|c| c.order);
            sorted
        })
    }

    /// Replace the entire tree (e.g., from an upstream server signal).
    pub fn set_tree(&self, new_tree: PaneNode<D>) {
        let max_id = new_tree
            .leaf_ids()
            .into_iter()
            .map(|id| id.0)
            .max()
            .unwrap_or(0);
        self.next_id.set(max_id + 1);
        self.tree.set(new_tree);
    }
}
