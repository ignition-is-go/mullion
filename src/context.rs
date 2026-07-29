use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use leptos::prelude::*;
use send_wrapper::SendWrapper;

use crate::activity::{ActivityDef, ActivityIcon, ActivityWithCategory, Category, CategoryMeta};
use crate::components::activity_bar::{ActivityBarBehavior, ActivityBarStyle};
use crate::components::drop_overlay::DropOverlayStyle;
use crate::components::mullion_root::MullionStyle;
use crate::components::pane_header::HeaderStyle;
use crate::components::pane_view::PaneStyle;
use crate::components::split_handle::SplitHandleStyle;
use crate::drag::DragPayload;
use crate::events::PaneEvent;
use crate::theme::MullionTheme;
use crate::tree::{
    collect_split_ratios, find_ratio, ActivityId, CategoryId, DropEdge, PaneData, PaneId, PaneNode,
    SplitDirection,
};

/// Host-provided per-pane chrome rendered in each pane's activity bar.
///
/// Unlike [`crate::activity::ActivityRender`] (a bare `fn` pointer that cannot
/// capture state), this is a boxed closure, so the host can close over app-level
/// signals — e.g. to render a session-color indicator that resolves the pane's
/// group/session from live server state. Called with the pane's id; returns the
/// chrome to mount in the activity bar's bottom action area.
pub type PaneAccessory = Arc<dyn Fn(PaneId) -> AnyView + Send + Sync>;

/// Host-provided per-pane bottom-border color (e.g. the pane's session color).
///
/// Returns a CSS color string for the pane, or `None` for no border. Called
/// reactively in the pane's render, so a closure that reads live signals (the
/// pane's group/session) updates the border when the session changes. mullion
/// owns the thickness/placement (a thin `border-bottom`); the host owns the color.
pub type PaneBorderColor = Arc<dyn Fn(PaneId) -> Option<String> + Send + Sync>;

/// Host predicate deciding whether a pane hides its activity bar, given the pane's
/// data. A pane with a hidden bar shows its content full-width and gets a small
/// hover-revealed control strip (split / close / move) instead, so it stays
/// manageable — useful for a pane dedicated to one thing (e.g. a video feed) whose
/// single-icon bar is just noise.
pub type PaneHideActivityBar<D> = Arc<dyn Fn(&D) -> bool + Send + Sync>;

/// Host predicate deciding whether a pane *auto-hides* its activity bar, given the
/// pane's data. A pane matching this keeps its full activity bar (unlike
/// [`PaneHideActivityBar`]) but tucks it off the pane's left edge; the bar is
/// invisible until the cursor reaches that edge, then slides in over the content
/// as an overlay — so a pane dedicated to one visual (e.g. a video feed) is
/// unobstructed until you reach for the bar. `None` = every pane keeps a pinned,
/// always-visible bar.
pub type PaneAutoHideActivityBar<D> = Arc<dyn Fn(&D) -> bool + Send + Sync>;

/// Host hook that mints a pane for an activity dragged out of the activity bar.
///
/// Mullion owns the layout tree but cannot invent a pane: only the host knows
/// how to allocate a [`PaneId`] and build a `D` (in a persisted app, that
/// usually means creating the pane entity server-side, or minting a client-side
/// id to be reconciled). So drop-to-create asks the host for those two values
/// and then does the tree surgery itself.
///
/// Called with the dragged activity, the destination pane the drop landed on,
/// and the edge — the destination lets the host inherit context from the
/// neighbouring pane (project, session, …). Return `None` to refuse the drop,
/// leaving the layout untouched.
///
/// Without this hook the feature is off: activities are not draggable at all,
/// so there is no affordance that silently does nothing.
pub type PaneFactory<D> =
    Arc<dyn Fn(&ActivityId, &PaneId, DropEdge) -> Option<(PaneId, D)> + Send + Sync>;

/// The reactive store for the mullion pane system.
///
/// Provided via Leptos context at `<MullionRoot>`. The consuming app interacts
/// with panes through this context.
#[derive(Clone)]
pub struct MullionContext<D: PaneData> {
    /// The reactive pane tree. Structural mutations (split / close / move /
    /// direction change / data / active_activity) notify subscribers here.
    ///
    /// Ratio updates DO NOT notify this signal — they go through the
    /// separate `ratios` map so that resize drags don't invalidate the
    /// whole rendered tree. The tree's inline `ratio: f64` fields are
    /// kept in sync via `update_untracked` for persistence snapshots.
    pub tree: RwSignal<PaneNode<D>>,
    /// Per-split ratio signals, keyed by each split's `split_key` — the
    /// first leaf id under its `second` subtree. See
    /// [`PaneNode::set_split_ratio`] for why we key splits this way.
    ///
    /// Seeded from the tree on construction and re-seeded after every
    /// structural mutation. `resize_split` writes only to these signals
    /// (plus an untracked tree write), so ratio updates re-render only
    /// the affected split's descendants' `Rect` memos.
    ///
    /// Uses `ArcRwSignal` (not `RwSignal`) so the signals' lifetimes are
    /// tied to the map itself, not to whatever reactive scope happened
    /// to be active when the signal was first accessed. Otherwise a
    /// signal created lazily during a structural re-render would be
    /// disposed along with that transient scope.
    pub(crate) ratios: StoredValue<HashMap<PaneId, ArcRwSignal<f64>>>,
    /// All registered activities (flattened, with category ids).
    pub(crate) activities: StoredValue<Vec<ActivityWithCategory<D>>>,
    /// Category metadata (without activities), sorted by order.
    pub(crate) categories: StoredValue<Vec<CategoryMeta>>,
    /// Event sink — write end. Every mutation pushes an event here.
    event_tx: StoredValue<Box<dyn Fn(PaneEvent<D>) + Send + Sync>>,
    /// The pane the mouse is currently over.
    pub focused_pane: RwSignal<Option<PaneId>>,
    /// What is currently being dragged, if anything — an existing pane being
    /// relocated, or an activity being placed as a new pane. See
    /// [`DragPayload`]; for the common narrow questions use
    /// [`Self::dragging_pane`] / [`Self::dragging_activity`].
    pub drag: RwSignal<Option<DragPayload>>,
    /// Global color theme.
    pub theme: MullionTheme,
    /// Resolved themes (captured at provider time so they work in reactive closures).
    pub mullion_style: MullionStyle,
    pub activity_bar_style: ActivityBarStyle,
    pub split_handle_style: SplitHandleStyle,
    pub pane_style: PaneStyle,
    pub drop_overlay_style: DropOverlayStyle,
    pub header_style: HeaderStyle,
    /// Activity bar interaction options (resolved at provider time).
    pub activity_bar_behavior: ActivityBarBehavior,
    /// Optional app icon displayed at the top of every activity bar.
    pub app_icon: Option<ActivityIcon>,
    /// Optional host-provided per-pane chrome rendered in the activity bar's
    /// bottom action area (e.g. a session-color indicator/switcher).
    pub pane_accessory: Option<PaneAccessory>,
    /// Optional host-provided per-pane bottom-border color (e.g. session color).
    pub pane_border_color: Option<PaneBorderColor>,
    /// Whether each pane renders its header band (the active activity's title).
    /// `false` suppresses it entirely — useful when the host drives navigation
    /// itself and the title is redundant. Defaults to `true`.
    pub show_pane_header: bool,
    /// Optional host predicate: panes for which it returns `true` hide their
    /// activity bar (and get hover controls instead). `None` = every pane keeps
    /// its bar.
    pub hide_activity_bar: Option<PaneHideActivityBar<D>>,
    /// Optional host predicate: panes for which it returns `true` auto-hide their
    /// activity bar (kept, but tucked off the left edge and revealed on edge-hover).
    /// `None` = every pane's bar is pinned/always-visible.
    pub auto_hide_activity_bar: Option<PaneAutoHideActivityBar<D>>,
    /// Optional host hook that mints a pane for an activity dragged out of the
    /// activity bar. `None` = activities are not draggable (feature off).
    pub new_pane: Option<PaneFactory<D>>,
    /// DOM element refs for each leaf pane (for positioning overlays, tooltips, etc.).
    pane_elements: Arc<Mutex<HashMap<PaneId, SendWrapper<web_sys::HtmlElement>>>>,
}

impl<D: PaneData + Send + Sync> MullionContext<D> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        initial_tree: PaneNode<D>,
        categories: Vec<Category<D>>,
        floating_activities: Vec<ActivityDef<D>>,
        event_handler: impl Fn(PaneEvent<D>) + Send + Sync + 'static,
        theme: MullionTheme,
        mullion_style: MullionStyle,
        activity_bar_style: ActivityBarStyle,
        split_handle_style: SplitHandleStyle,
        pane_style: PaneStyle,
        drop_overlay_style: DropOverlayStyle,
        header_style: HeaderStyle,
        activity_bar_behavior: ActivityBarBehavior,
        app_icon: Option<ActivityIcon>,
        pane_accessory: Option<PaneAccessory>,
        pane_border_color: Option<PaneBorderColor>,
        show_pane_header: bool,
        hide_activity_bar: Option<PaneHideActivityBar<D>>,
        auto_hide_activity_bar: Option<PaneAutoHideActivityBar<D>>,
        new_pane: Option<PaneFactory<D>>,
    ) -> Self {
        // Flatten categories into metadata + activities with category ids. Floating
        // activities (registered outside any category) carry `category: None`.
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
                    category: Some(cat_metas.last().unwrap().id.clone()),
                });
            }
        }
        for act in floating_activities {
            all_activities.push(ActivityWithCategory {
                def: act,
                category: None,
            });
        }

        cat_metas.sort_by_key(|c| c.order);

        // Seed the ratio signal map from the initial tree's splits.
        let mut initial_ratios = Vec::new();
        collect_split_ratios(&initial_tree, &mut initial_ratios);
        let ratio_map: HashMap<PaneId, ArcRwSignal<f64>> = initial_ratios
            .into_iter()
            .map(|(k, r)| (k, ArcRwSignal::new(r)))
            .collect();

        MullionContext {
            tree: RwSignal::new(initial_tree),
            ratios: StoredValue::new(ratio_map),
            activities: StoredValue::new(all_activities),
            categories: StoredValue::new(cat_metas),
            event_tx: StoredValue::new(Box::new(event_handler)),
            focused_pane: RwSignal::new(None),
            drag: RwSignal::new(None),
            theme,
            mullion_style,
            activity_bar_style,
            split_handle_style,
            pane_style,
            drop_overlay_style,
            header_style,
            activity_bar_behavior,
            app_icon,
            pane_accessory,
            pane_border_color,
            show_pane_header,
            hide_activity_bar,
            auto_hide_activity_bar,
            new_pane,
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

    /// Get-or-create the ratio signal for a split, keyed by the first leaf id
    /// under its `second` subtree (see [`PaneNode::set_split_ratio`]).
    ///
    /// Returns a signal initialized from the tree if the entry was missing.
    /// Used by the renderer to bind a split's flex-basis reactively. The
    /// check-and-insert is performed atomically inside `try_update_value`
    /// so concurrent callers always observe the same signal instance.
    pub(crate) fn ratio_signal(&self, split_key: &PaneId) -> ArcRwSignal<f64> {
        // Fast path: the map already has an entry — avoid allocating a
        // new signal just to throw it away.
        if let Some(sig) = self.ratios.with_value(|m| m.get(split_key).cloned()) {
            return sig;
        }
        let initial = self
            .tree
            .with_untracked(|t| find_ratio(t, split_key))
            .unwrap_or(0.5);
        self.ratios
            .try_update_value(|m| {
                m.entry(split_key.clone())
                    .or_insert_with(|| ArcRwSignal::new(initial))
                    .clone()
            })
            // `try_update_value` only returns None if the StoredValue is
            // disposed, which shouldn't happen while the context is alive;
            // fall back to an unattached signal in that pathological case.
            .unwrap_or_else(|| ArcRwSignal::new(initial))
    }

    /// Re-sync the ratio map to the current tree after a structural change.
    ///
    /// Adds missing entries, drops entries for splits that no longer exist,
    /// and updates existing signals' values to match the tree. Never creates
    /// a new signal for a still-existing split so that subscribers keep their
    /// reference live across structural ops.
    fn reseed_ratios(&self) {
        let mut collected = Vec::new();
        self.tree
            .with_untracked(|t| collect_split_ratios(t, &mut collected));
        let keys: std::collections::HashSet<PaneId> =
            collected.iter().map(|(k, _)| k.clone()).collect();

        self.ratios.update_value(|m| {
            m.retain(|k, _| keys.contains(k));
            for (key, ratio) in &collected {
                match m.get(key) {
                    Some(existing) => {
                        if (existing.get_untracked() - ratio).abs() > f64::EPSILON {
                            existing.set(*ratio);
                        }
                    }
                    None => {
                        m.insert(key.clone(), ArcRwSignal::new(*ratio));
                    }
                }
            }
        });
    }

    /// Split a pane. The consumer provides the new pane's id.
    pub fn split_pane(
        &self,
        target: &PaneId,
        direction: SplitDirection,
        new_id: PaneId,
        new_data: D,
    ) {
        self.tree.update(|tree| {
            tree.split(target, direction, new_id.clone(), new_data.clone());
        });
        self.reseed_ratios();
        self.emit(PaneEvent::Split {
            target: target.clone(),
            direction,
            new_id,
            new_data,
        });
        self.emit_tree_changed();
    }

    /// Close a pane. Returns the closed pane's data if found.
    pub fn close_pane(&self, id: &PaneId) -> Option<D> {
        let mut closed_data = None;
        self.tree.update(|tree| {
            closed_data = tree.close(id);
        });
        if let Some(ref data) = closed_data {
            self.reseed_ratios();
            self.emit(PaneEvent::Closed {
                id: id.clone(),
                data: data.clone(),
            });
            self.emit_tree_changed();
        }
        closed_data
    }

    /// Resize a split by its `split_key` (the first leaf id under the
    /// split's `second` subtree — see [`PaneNode::set_split_ratio`]).
    ///
    /// `ratio` is the fraction of the split's parent area given to the
    /// `first` subtree, clamped to `[0.1, 0.9]`.
    ///
    /// Writes go through two channels:
    /// 1. The per-split ratio `ArcRwSignal` — subscribed by the affected
    ///    leaves' rect memos, so their styles update.
    /// 2. The tree itself, via `update_untracked`, keeping the stored
    ///    `ratio` field in sync for persistence without notifying
    ///    structural subscribers.
    ///
    /// Calls with an unknown `split_key` are ignored (no events emitted,
    /// no signal created).
    ///
    /// On success, emits `PaneEvent::Resized` and `PaneEvent::TreeChanged`.
    pub fn resize_split(&self, split_key: &PaneId, ratio: f64) {
        if !ratio.is_finite() {
            return;
        }
        let clamped = ratio.clamp(0.1, 0.9);
        let mut matched = false;
        self.tree.update_untracked(|tree| {
            matched = tree.set_split_ratio(split_key, clamped);
        });
        if !matched {
            return;
        }
        let sig = self.ratio_signal(split_key);
        sig.set(clamped);
        self.emit(PaneEvent::Resized {
            split_key: split_key.clone(),
            ratio: clamped,
        });
        self.emit_tree_changed();
    }

    /// Change the split direction of a pane's parent.
    pub fn change_split_direction(&self, pane: &PaneId, direction: SplitDirection) {
        self.tree.update(|tree| {
            tree.change_direction(pane, direction);
        });
        self.emit(PaneEvent::DirectionChanged {
            pane: pane.clone(),
            direction,
        });
        self.emit_tree_changed();
    }

    /// Move a pane to a new position relative to a destination pane.
    pub fn move_pane(&self, source: &PaneId, destination: &PaneId, edge: DropEdge) {
        let mut success = false;
        self.tree.update(|tree| {
            success = tree.move_pane(source, destination, edge);
        });
        if success {
            self.reseed_ratios();
            self.emit(PaneEvent::Moved {
                source: source.clone(),
                destination: destination.clone(),
                edge,
            });
            self.emit_tree_changed();
        }
    }

    /// The pane currently being relocated, or `None` if nothing is being
    /// dragged *or* the drag is a new-activity drag. Reactive.
    pub fn dragging_pane(&self) -> Option<PaneId> {
        self.drag.get().and_then(|p| p.pane().cloned())
    }

    /// The activity currently being dragged out of an activity bar, or `None` if
    /// nothing is being dragged *or* the drag is a pane move. Reactive.
    pub fn dragging_activity(&self) -> Option<ActivityId> {
        self.drag.get().and_then(|p| p.activity().cloned())
    }

    /// Place a dragged activity into the layout as a new pane, beside
    /// `destination` at `edge`.
    ///
    /// Asks the host's `new_pane` hook ([`PaneFactory`]) for the new pane's id
    /// and data, then does the tree surgery. Does nothing if no hook is
    /// installed or the hook returns `None` (a refused drop leaves the layout
    /// untouched).
    ///
    /// On success emits `PaneEvent::ActivityDropped` then
    /// `PaneEvent::TreeChanged`.
    pub fn drop_activity(&self, activity: &ActivityId, destination: &PaneId, edge: DropEdge) {
        let Some(factory) = self.new_pane.as_ref() else {
            return;
        };
        let Some((new_id, new_data)) = factory(activity, destination, edge) else {
            return;
        };

        let mut inserted = false;
        self.tree.update(|tree| {
            inserted = tree.insert_leaf(
                destination,
                edge,
                new_id.clone(),
                new_data.clone(),
                Some(activity.clone()),
            );
        });
        if !inserted {
            return;
        }

        self.reseed_ratios();
        self.emit(PaneEvent::ActivityDropped {
            activity: activity.clone(),
            destination: destination.clone(),
            edge,
            new_id,
            new_data,
        });
        self.emit_tree_changed();
    }

    /// Set the active activity for a pane.
    pub fn set_active_activity(&self, pane: &PaneId, activity: Option<ActivityId>) {
        let act_clone = activity.clone();
        self.tree.update(|tree| {
            if let Some(PaneNode::Leaf {
                active_activity, ..
            }) = tree.find_mut(pane)
            {
                *active_activity = act_clone;
            }
        });
        self.emit(PaneEvent::ActivityChanged {
            pane: pane.clone(),
            activity,
        });
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

    /// Look up an activity's category id. `None` for a floating activity (or an
    /// unknown id).
    pub fn activity_category(&self, activity_id: &ActivityId) -> Option<CategoryId> {
        self.activities.with_value(|acts| {
            acts.iter()
                .find(|a| a.def.id == *activity_id)
                .and_then(|a| a.category.clone())
        })
    }

    /// Update a single pane's data without replacing the whole tree.
    pub fn update_pane_data(&self, pane: &PaneId, new_data: D) {
        self.tree.update(|tree| {
            if let Some(PaneNode::Leaf { data, .. }) = tree.find_mut(pane) {
                *data = new_data;
            }
        });
        self.emit_tree_changed();
    }

    /// Get a pane's current data.
    pub fn pane_data(&self, pane: &PaneId) -> Option<D> {
        self.tree.with_untracked(|tree| match tree.find(pane) {
            Some(PaneNode::Leaf { data, .. }) => Some(data.clone()),
            _ => None,
        })
    }

    /// Update the tree with a closure. Emits a TreeChanged event.
    pub fn update_tree(&self, f: impl FnOnce(&mut PaneNode<D>)) {
        self.tree.update(f);
        self.reseed_ratios();
        self.emit_tree_changed();
    }

    /// Replace the entire tree (e.g., from an upstream server signal).
    pub fn set_tree(&self, new_tree: PaneNode<D>) {
        self.tree.set(new_tree);
        self.reseed_ratios();
    }

    /// Register a pane's DOM element (called internally by PaneView on mount).
    pub(crate) fn register_pane_element(&self, id: PaneId, el: web_sys::HtmlElement) {
        self.pane_elements
            .lock()
            .unwrap()
            .insert(id, SendWrapper::new(el));
    }

    /// Get the DOM element for a pane. Use this to position overlays,
    /// tooltips, or anything relative to a specific pane.
    pub fn pane_element(&self, id: PaneId) -> Option<web_sys::HtmlElement> {
        self.pane_elements
            .lock()
            .unwrap()
            .get(&id)
            .map(|w| w.clone().take())
    }

    /// Get the bounding rect for a pane.
    pub fn pane_rect(&self, id: PaneId) -> Option<web_sys::DomRect> {
        self.pane_elements
            .lock()
            .unwrap()
            .get(&id)
            .map(|el| el.get_bounding_client_rect())
    }
}
