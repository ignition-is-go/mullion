use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use leptos::prelude::*;
use send_wrapper::SendWrapper;

use crate::activity::{ActivityIcon, ActivityWithCategory, Category, CategoryMeta};
use crate::events::PaneEvent;
use crate::theme::{ActivityBarTheme, DropOverlayTheme, MullionTheme, PaneTheme, SplitHandleTheme};
use crate::tree::{ActivityId, CategoryId, DropEdge, PaneData, PaneId, PaneNode, SplitDirection};

/// The reactive store for the mullion pane system.
///
/// Provided via Leptos context at `<MullionRoot>`. The consuming app interacts
/// with panes through this context.
#[derive(Clone)]
pub struct MullionContext<D: PaneData> {
    /// The reactive pane tree. Can be read to render, updated for mutations.
    pub tree: RwSignal<PaneNode<D>>,
    /// All registered activities (flattened, with category ids).
    pub(crate) activities: StoredValue<Vec<ActivityWithCategory<D>>>,
    /// Category metadata (without activities), sorted by order.
    pub(crate) categories: StoredValue<Vec<CategoryMeta>>,
    /// Event sink — write end. Every mutation pushes an event here.
    event_tx: StoredValue<Box<dyn Fn(PaneEvent<D>) + Send + Sync>>,
    /// Counter for generating unique PaneIds.
    next_id: RwSignal<u64>,
    /// The pane the mouse is currently over.
    pub focused_pane: RwSignal<Option<PaneId>>,
    /// Pane currently being dragged (for move operations).
    pub dragging_pane: RwSignal<Option<PaneId>>,
    /// Resolved themes (captured at provider time so they work in reactive closures).
    pub mullion_theme: MullionTheme,
    pub activity_bar_theme: ActivityBarTheme,
    pub split_handle_theme: SplitHandleTheme,
    pub pane_theme: PaneTheme,
    pub drop_overlay_theme: DropOverlayTheme,
    /// Optional app icon displayed at the top of every activity bar.
    pub app_icon: Option<ActivityIcon>,
    /// DOM element refs for each leaf pane (for positioning overlays, tooltips, etc.).
    pane_elements: Arc<Mutex<HashMap<PaneId, SendWrapper<web_sys::HtmlElement>>>>,
}

impl<D: PaneData + Send + Sync> MullionContext<D> {
    pub fn new(
        initial_tree: PaneNode<D>,
        categories: Vec<Category<D>>,
        event_handler: impl Fn(PaneEvent<D>) + Send + Sync + 'static,
        mullion_theme: MullionTheme,
        activity_bar_theme: ActivityBarTheme,
        split_handle_theme: SplitHandleTheme,
        pane_theme: PaneTheme,
        drop_overlay_theme: DropOverlayTheme,
        app_icon: Option<ActivityIcon>,
    ) -> Self {
        let max_id = initial_tree
            .leaf_ids()
            .into_iter()
            .map(|id| id.0)
            .max()
            .unwrap_or(0);

        // Flatten categories into metadata + activities with category ids
        let mut cat_metas = Vec::new();
        let mut all_activities = Vec::new();

        for cat in categories {
            cat_metas.push(CategoryMeta {
                id: cat.id,
                name: cat.name,
                order: cat.order,
                icon: cat.icon,
                color: cat.color,
            });
            for act in cat.activities {
                all_activities.push(ActivityWithCategory {
                    def: act,
                    category: cat_metas.last().unwrap().id,
                });
            }
        }

        cat_metas.sort_by_key(|c| c.order);

        MullionContext {
            tree: RwSignal::new(initial_tree),
            activities: StoredValue::new(all_activities),
            categories: StoredValue::new(cat_metas),
            event_tx: StoredValue::new(Box::new(event_handler)),
            next_id: RwSignal::new(max_id + 1),
            focused_pane: RwSignal::new(None),
            dragging_pane: RwSignal::new(None),
            mullion_theme,
            activity_bar_theme,
            split_handle_theme,
            pane_theme,
            drop_overlay_theme,
            app_icon,
            pane_elements: Arc::new(Mutex::new(HashMap::new())),
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
    /// Returns (activity_def, category_id) pairs.
    pub fn activities_for_pane(&self, data: &D) -> Vec<ActivityWithCategory<D>> {
        self.activities.with_value(|acts| {
            acts.iter()
                .filter(|a| (a.def.filter)(data))
                .cloned()
                .collect()
        })
    }

    /// Get categories sorted by order.
    pub fn sorted_categories(&self) -> Vec<CategoryMeta> {
        self.categories.with_value(|cats| cats.clone())
    }

    /// Look up an activity's category id.
    pub fn activity_category(&self, activity_id: ActivityId) -> Option<CategoryId> {
        self.activities.with_value(|acts| {
            acts.iter()
                .find(|a| a.def.id == activity_id)
                .map(|a| a.category)
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

    /// Register a pane's DOM element (called internally by PaneView on mount).
    pub(crate) fn register_pane_element(&self, id: PaneId, el: web_sys::HtmlElement) {
        self.pane_elements.lock().unwrap().insert(id, SendWrapper::new(el));
    }

    /// Get the DOM element for a pane. Use this to position overlays,
    /// tooltips, or anything relative to a specific pane.
    pub fn pane_element(&self, id: PaneId) -> Option<web_sys::HtmlElement> {
        self.pane_elements.lock().unwrap().get(&id).map(|w| w.clone().take())
    }

    /// Get the bounding rect for a pane.
    pub fn pane_rect(&self, id: PaneId) -> Option<web_sys::DomRect> {
        self.pane_elements.lock().unwrap().get(&id).map(|el| el.get_bounding_client_rect())
    }
}
