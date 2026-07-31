use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use leptos::prelude::*;
use send_wrapper::SendWrapper;

use crate::activity::{ActivityIcon, ActivityNode, ActivityWithCategory, CategoryMeta};
use crate::components::activity_bar::{ActivityBarBehavior, ActivityBarEdge, ActivityBarStyle};
use crate::components::drop_overlay::DropOverlayStyle;
use crate::components::mullion_root::MullionStyle;
use crate::components::pane_header::HeaderStyle;
use crate::components::pane_view::PaneStyle;
use crate::components::split_handle::SplitHandleStyle;
use crate::drag::DragPayload;
use crate::events::PaneEvent;
use crate::focus::PaneFocusBehavior;
use crate::theme::MullionTheme;
use crate::tree::{
    collect_split_ratios, directional_neighbor, find_ratio, resize_boundary, ActivityId,
    CategoryId, DropEdge, PaneData, PaneDirection, PaneId, PaneLayout, PaneNode, PaneRotation,
    SplitDirection,
};

/// Host-provided per-pane chrome rendered in each pane's activity bar.
///
/// Unlike [`crate::activity::ActivityRender`] (a bare `fn` pointer that cannot
/// capture state), this is a boxed closure, so the host can close over app-level
/// signals — e.g. to render a session-color indicator that resolves the pane's
/// group/session from live server state. Called with the pane's id; returns the
/// chrome to mount in the activity bar's secondary action area (bottom when
/// vertical, right when horizontal).
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
/// [`PaneHideActivityBar`]) but tucks it off its configured edge. The bar is
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
    /// The registered item tree, in render order. The activity bar walks this;
    /// `activities` and `categories` are flattened views of it for lookup.
    pub(crate) items: StoredValue<Vec<ActivityNode<D>>>,
    /// A second item tree anchored opposite the primary tree: at the bottom of
    /// a vertical bar or the right of a horizontal one. Same shape and same
    /// behaviour as `items` — activities, categories, arbitrary nesting. This
    /// is where settings-like entries go rather than trailing the main group.
    pub(crate) bottom_items: StoredValue<Vec<ActivityNode<D>>>,
    pub(crate) activities: StoredValue<Vec<ActivityWithCategory<D>>>,
    /// Category metadata (without children), in registration (pre-order) order.
    pub(crate) categories: StoredValue<Vec<CategoryMeta>>,
    /// Event sink — write end. Every mutation pushes an event here.
    event_tx: StoredValue<Box<dyn Fn(PaneEvent<D>) + Send + Sync>>,
    /// The pane that receives focus-relative commands.
    ///
    /// Pointer interaction updates this according to [`Self::focus_behavior`];
    /// applications may also read or set it directly, or use
    /// [`Self::focus_pane`].
    pub focused_pane: RwSignal<Option<PaneId>>,
    /// Whether hovering or clicking acquires pane focus.
    pub focus_behavior: PaneFocusBehavior,
    /// The pane temporarily occupying the full Mullion viewport.
    ///
    /// This is presentation state: toggling zoom does not rewrite or emit a
    /// persisted tree snapshot.
    pub zoomed_pane: RwSignal<Option<PaneId>>,
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
    /// Edge used by each pane's activity bar.
    pub activity_bar_edge: ActivityBarEdge,
    /// Optional app icon displayed at the leading edge of every activity bar.
    pub app_icon: Option<ActivityIcon>,
    /// Optional host-provided per-pane chrome rendered in the activity bar's
    /// secondary action area (e.g. a session-color indicator/switcher).
    pub pane_accessory: Option<PaneAccessory>,
    /// Optional host-provided per-pane bottom-border color (e.g. session color).
    pub pane_border_color: Option<PaneBorderColor>,
    /// Host slot rendered immediately *before* the secondary activity group.
    pub bottom_leading: Option<PaneAccessory>,
    /// Host slot rendered immediately *after* the secondary activity group,
    /// before `pane_accessory` and the built-in split/close controls.
    pub bottom_trailing: Option<PaneAccessory>,
    /// Whether each pane renders its header band (the active activity's title).
    /// `false` suppresses it entirely — useful when the host drives navigation
    /// itself and the title is redundant. Defaults to `true`.
    pub show_pane_header: bool,
    /// Optional host predicate: panes for which it returns `true` hide their
    /// activity bar (and get hover controls instead). `None` = every pane keeps
    /// its bar.
    pub hide_activity_bar: Option<PaneHideActivityBar<D>>,
    /// Optional host predicate: panes for which it returns `true` auto-hide their
    /// activity bar (kept, but tucked off its configured edge and revealed on
    /// edge-hover).
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
        items: Vec<ActivityNode<D>>,
        bottom_items: Vec<ActivityNode<D>>,
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
        bottom_leading: Option<PaneAccessory>,
        bottom_trailing: Option<PaneAccessory>,
    ) -> Self {
        // Flatten the item tree for lookup, keeping the tree itself for render.
        // Both come out in pre-order, so "sorted" is just registration order —
        // there is no `order` field to sort by any more.
        let (cat_metas, all_activities) = flatten_groups(&items, &bottom_items);

        // Seed the ratio signal map from the initial tree's splits.
        let mut initial_ratios = Vec::new();
        collect_split_ratios(&initial_tree, &mut initial_ratios);
        let ratio_map: HashMap<PaneId, ArcRwSignal<f64>> = initial_ratios
            .into_iter()
            .map(|(k, r)| (k, ArcRwSignal::new(r)))
            .collect();
        let initial_focus = initial_tree.leaf_ids().into_iter().next();

        MullionContext {
            tree: RwSignal::new(initial_tree),
            ratios: StoredValue::new(ratio_map),
            items: StoredValue::new(items),
            bottom_items: StoredValue::new(bottom_items),
            activities: StoredValue::new(all_activities),
            categories: StoredValue::new(cat_metas),
            event_tx: StoredValue::new(Box::new(event_handler)),
            focused_pane: RwSignal::new(initial_focus),
            focus_behavior: PaneFocusBehavior::default(),
            zoomed_pane: RwSignal::new(None),
            drag: RwSignal::new(None),
            theme,
            mullion_style,
            activity_bar_style,
            split_handle_style,
            pane_style,
            drop_overlay_style,
            header_style,
            activity_bar_behavior,
            activity_bar_edge: ActivityBarEdge::default(),
            app_icon,
            pane_accessory,
            pane_border_color,
            bottom_leading,
            bottom_trailing,
            show_pane_header,
            hide_activity_bar,
            auto_hide_activity_bar,
            new_pane,
            pane_elements: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Set the activity bar edge used by every pane in this context.
    ///
    /// [`Self::new`] preserves the historic left-edge default. This builder lets
    /// custom context constructors choose another edge without
    /// changing that long-standing constructor signature.
    pub fn with_activity_bar_edge(mut self, edge: ActivityBarEdge) -> Self {
        self.activity_bar_edge = edge;
        self
    }

    /// Configure how pointer interaction acquires pane focus.
    pub fn with_focus_behavior(mut self, behavior: PaneFocusBehavior) -> Self {
        self.focus_behavior = behavior;
        self
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

    fn reconcile_interaction_state(&self, preferred_focus: Option<PaneId>) {
        let ids = self.tree.with_untracked(PaneNode::leaf_ids);
        let current = self.focused_pane.get_untracked();
        let next = current
            .filter(|id| ids.contains(id))
            .or_else(|| preferred_focus.filter(|id| ids.contains(id)))
            .or_else(|| ids.first().cloned());
        self.focused_pane.set(next);

        if self
            .zoomed_pane
            .get_untracked()
            .as_ref()
            .is_some_and(|id| !ids.contains(id))
        {
            self.zoomed_pane.set(None);
        }
    }

    /// Pane ids in the layout's stable traversal order.
    pub fn pane_ids(&self) -> Vec<PaneId> {
        self.tree.with_untracked(PaneNode::leaf_ids)
    }

    /// Resolve the currently focused pane, repairing stale focus after an
    /// upstream tree replacement by selecting the first remaining pane.
    pub fn focused_pane_id(&self) -> Option<PaneId> {
        let current = self.focused_pane.get_untracked();
        if current
            .as_ref()
            .is_some_and(|id| self.tree.with_untracked(|tree| tree.contains(id)))
        {
            return current;
        }
        let first = self
            .tree
            .with_untracked(|tree| tree.leaf_ids().into_iter().next());
        self.focused_pane.set(first.clone());
        first
    }

    /// Focus a pane by id. Returns `false` when the pane is not in this layout.
    pub fn focus_pane(&self, pane: &PaneId) -> bool {
        if !self.tree.with_untracked(|tree| tree.contains(pane)) {
            return false;
        }
        self.focused_pane.set(Some(pane.clone()));
        // A zoomed layout remains zoomed while focus commands switch which pane
        // occupies the viewport, matching terminal multiplexer behavior.
        if self.zoomed_pane.get_untracked().is_some() {
            self.zoomed_pane.set(Some(pane.clone()));
        }
        true
    }

    /// Focus the visually nearest pane in a direction.
    pub fn focus_neighbor(&self, direction: PaneDirection) -> bool {
        let Some(current) = self.focused_pane_id() else {
            return false;
        };
        let next = self.tree.with_untracked(|tree| {
            directional_neighbor(tree, &current, direction, |key| {
                self.ratio_signal(key).get_untracked()
            })
        });
        next.as_ref().is_some_and(|pane| self.focus_pane(pane))
    }

    /// Focus the next or previous pane in traversal order, wrapping at either
    /// end. A positive offset moves forward; a negative offset moves backward.
    pub fn cycle_focus(&self, offset: isize) -> bool {
        let ids = self.pane_ids();
        if ids.is_empty() {
            return false;
        }
        let current = self.focused_pane_id();
        let index = current
            .as_ref()
            .and_then(|current| ids.iter().position(|id| id == current))
            .unwrap_or(0);
        let next = (index as isize + offset).rem_euclid(ids.len() as isize) as usize;
        self.focus_pane(&ids[next])
    }

    /// Focus a pane by zero-based traversal index.
    pub fn focus_pane_at(&self, index: usize) -> bool {
        self.pane_ids()
            .get(index)
            .is_some_and(|pane| self.focus_pane(pane))
    }

    /// Toggle whether the focused pane occupies the full Mullion viewport.
    ///
    /// Zoom is local presentation state and does not emit a pane event.
    pub fn toggle_zoom(&self) -> bool {
        let Some(focused) = self.focused_pane_id() else {
            return false;
        };
        if self.zoomed_pane.get_untracked().as_ref() == Some(&focused) {
            self.zoomed_pane.set(None);
        } else {
            self.zoomed_pane.set(Some(focused));
        }
        true
    }

    /// Split a pane. The consumer provides the new pane's id.
    pub fn split_pane(
        &self,
        target: &PaneId,
        direction: SplitDirection,
        new_id: PaneId,
        new_data: D,
    ) {
        self.try_split_pane(target, direction, new_id, new_data);
    }

    /// Fallible form of [`Self::split_pane`].
    ///
    /// Returns `false` for an unknown target or a duplicate new pane id and
    /// emits nothing in either case.
    pub fn try_split_pane(
        &self,
        target: &PaneId,
        direction: SplitDirection,
        new_id: PaneId,
        new_data: D,
    ) -> bool {
        if self
            .tree
            .with_untracked(|tree| !tree.contains(target) || tree.contains(&new_id))
        {
            return false;
        }
        let mut split = false;
        self.tree.update(|tree| {
            split = tree.split(target, direction, new_id.clone(), new_data.clone());
        });
        if !split {
            return false;
        }
        self.reseed_ratios();
        self.emit(PaneEvent::Split {
            target: target.clone(),
            direction,
            new_id: new_id.clone(),
            new_data,
        });
        self.emit_tree_changed();
        self.focus_pane(&new_id);
        true
    }

    /// Close a pane. Returns the closed pane's data if found.
    pub fn close_pane(&self, id: &PaneId) -> Option<D> {
        let ids = self.pane_ids();
        let preferred_focus = ids.iter().position(|pane| pane == id).and_then(|index| {
            ids.get(index + 1)
                .or_else(|| index.checked_sub(1).and_then(|previous| ids.get(previous)))
                .cloned()
        });
        let mut closed_data = None;
        self.tree.update(|tree| {
            closed_data = tree.close(id);
        });
        if let Some(ref data) = closed_data {
            self.reseed_ratios();
            self.reconcile_interaction_state(preferred_focus);
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
        self.try_change_split_direction(pane, direction);
    }

    /// Fallible form of [`Self::change_split_direction`].
    pub fn try_change_split_direction(&self, pane: &PaneId, direction: SplitDirection) -> bool {
        let mut changed = false;
        self.tree.update(|tree| {
            changed = tree.change_direction(pane, direction);
        });
        if !changed {
            return false;
        }
        self.emit(PaneEvent::DirectionChanged {
            pane: pane.clone(),
            direction,
        });
        self.emit_tree_changed();
        true
    }

    /// Move a pane to a new position relative to a destination pane.
    pub fn move_pane(&self, source: &PaneId, destination: &PaneId, edge: DropEdge) {
        self.try_move_pane(source, destination, edge);
    }

    /// Fallible form of [`Self::move_pane`].
    pub fn try_move_pane(&self, source: &PaneId, destination: &PaneId, edge: DropEdge) -> bool {
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
        success
    }

    /// Find the visually nearest pane to `pane` in a direction.
    pub fn pane_neighbor(&self, pane: &PaneId, direction: PaneDirection) -> Option<PaneId> {
        self.tree.with_untracked(|tree| {
            directional_neighbor(tree, pane, direction, |key| {
                self.ratio_signal(key).get_untracked()
            })
        })
    }

    /// Swap two panes without changing the split topology.
    pub fn swap_panes(&self, first: &PaneId, second: &PaneId) -> bool {
        let mut swapped = false;
        self.tree.update(|tree| {
            swapped = tree.swap_panes(first, second);
        });
        if swapped {
            self.reseed_ratios();
            self.emit_tree_changed();
        }
        swapped
    }

    /// Rotate every pane through the existing layout slots.
    pub fn rotate_panes(&self, rotation: PaneRotation) -> bool {
        let mut rotated = false;
        self.tree.update(|tree| {
            rotated = tree.rotate_panes(rotation);
        });
        if rotated {
            self.reseed_ratios();
            self.emit_tree_changed();
        }
        rotated
    }

    /// Reset every split ratio to an equal half.
    pub fn balance_splits(&self) -> bool {
        let mut count = 0;
        self.tree.update_untracked(|tree| {
            count = tree.balance_splits();
        });
        if count == 0 {
            return false;
        }
        self.reseed_ratios();
        let mut splits = Vec::new();
        self.tree
            .with_untracked(|tree| collect_split_ratios(tree, &mut splits));
        for (split_key, ratio) in splits {
            self.emit(PaneEvent::Resized { split_key, ratio });
        }
        self.emit_tree_changed();
        true
    }

    /// Rebuild the pane tree using a standard layout.
    pub fn apply_layout(&self, layout: PaneLayout) -> bool {
        let focused = self.focused_pane_id();
        let mut applied = false;
        self.tree.update(|tree| {
            applied = tree.apply_layout(layout, focused.as_ref());
        });
        if applied {
            self.reseed_ratios();
            self.emit_tree_changed();
        }
        applied
    }

    /// Grow the pane toward its nearest boundary in `direction` by `amount`.
    pub fn resize_pane_toward(&self, pane: &PaneId, direction: PaneDirection, amount: f64) -> bool {
        if !amount.is_finite() || amount <= 0.0 {
            return false;
        }
        let boundary = self
            .tree
            .with_untracked(|tree| resize_boundary(tree, pane, direction));
        let Some((split_key, sign)) = boundary else {
            return false;
        };
        let current = self.ratio_signal(&split_key).get_untracked();
        self.resize_split(&split_key, current + sign * amount);
        true
    }

    /// Toggle the direction of the split immediately containing `pane`.
    pub fn toggle_parent_split_direction(&self, pane: &PaneId) -> bool {
        let current = self
            .tree
            .with_untracked(|tree| tree.parent_split_direction(pane));
        let Some(current) = current else {
            return false;
        };
        let next = match current {
            SplitDirection::Horizontal => SplitDirection::Vertical,
            SplitDirection::Vertical => SplitDirection::Horizontal,
        };
        self.try_change_split_direction(pane, next)
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
            new_id: new_id.clone(),
            new_data,
        });
        self.emit_tree_changed();
        self.focus_pane(&new_id);
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

    /// An activity's ancestor categories, outermost first. Empty for a
    /// top-level activity, or an unknown id.
    ///
    /// The activity bar expands the whole chain when an activity becomes active,
    /// so a nested activity reveals itself rather than staying hidden inside
    /// collapsed ancestors.
    pub fn activity_ancestors(&self, activity_id: &ActivityId) -> Vec<CategoryId> {
        self.activities.with_value(|acts| {
            acts.iter()
                .find(|a| a.def.id == *activity_id)
                .map(|a| a.path.clone())
                .unwrap_or_default()
        })
    }

    /// A category's colour, or `None` for an unknown id.
    pub fn category_color(&self, category_id: &CategoryId) -> Option<String> {
        self.categories.with_value(|cats| {
            cats.iter()
                .find(|c| c.id == *category_id)
                .map(|c| c.color.clone())
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
        self.reconcile_interaction_state(None);
        self.emit_tree_changed();
    }

    /// Replace the entire tree (e.g., from an upstream server signal).
    pub fn set_tree(&self, new_tree: PaneNode<D>) {
        self.tree.set(new_tree);
        self.reseed_ratios();
        self.reconcile_interaction_state(None);
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

/// Flatten both activity groups into the lookup tables the rest of the crate
/// reads.
///
/// Both groups must land here, not just the top one: `PaneContent` resolves a
/// pane's active activity through `activities`, so a bottom activity missing
/// from it would render as "activity not found". Top group first, so the
/// flattened order matches the visual order down the bar.
fn flatten_groups<D: PaneData>(
    top: &[ActivityNode<D>],
    bottom: &[ActivityNode<D>],
) -> (Vec<CategoryMeta>, Vec<ActivityWithCategory<D>>) {
    let mut cats = Vec::new();
    let mut acts = Vec::new();
    flatten_items(top, &mut Vec::new(), &mut cats, &mut acts);
    flatten_items(bottom, &mut Vec::new(), &mut cats, &mut acts);
    debug_assert_unique_ids(&cats, &acts);
    (cats, acts)
}

/// Panics in debug builds if an activity or category id is registered twice.
///
/// Ids are the identity everything else resolves through, and every lookup takes
/// the first match: `PaneContent` picks a pane's activity with `.find()`,
/// category expansion state is keyed by `CategoryId`, and a persisted pane stores
/// only an `ActivityId` to be re-derived later. So a duplicate never errors — the
/// bar renders both entries, they select each other because they share an id, and
/// the pane shows whichever was registered first. A copy-pasted registration is
/// the usual way in, and two groups (`items` and `bottom_items`) make it easier
/// to hit, since the collision can span groups where no single list looks wrong.
///
/// `debug_assert` rather than a hard panic: registration is host data and a
/// release build should not die over it, while dev and test builds get the loud
/// failure at the point it is cheap to fix. Costs nothing in release.
fn debug_assert_unique_ids<D: PaneData>(cats: &[CategoryMeta], acts: &[ActivityWithCategory<D>]) {
    if cfg!(debug_assertions) {
        let mut seen = std::collections::HashSet::new();
        let dup_act: Vec<&str> = acts
            .iter()
            .filter(|a| !seen.insert(&a.def.id.0))
            .map(|a| a.def.id.0.as_str())
            .collect();
        debug_assert!(
            dup_act.is_empty(),
            "activity id registered more than once: {dup_act:?} — ids are resolved              by first match, so duplicates render as separate bar entries that              select each other"
        );

        let mut seen = std::collections::HashSet::new();
        let dup_cat: Vec<&str> = cats
            .iter()
            .filter(|c| !seen.insert(&c.id.0))
            .map(|c| c.id.0.as_str())
            .collect();
        debug_assert!(
            dup_cat.is_empty(),
            "category id registered more than once: {dup_cat:?} — expansion state              is keyed by id, so duplicates open and close together"
        );
    }
}

/// Walk one registered item tree, collecting a flat list of categories and of
/// activities-with-their-ancestor-path.
///
/// `path` is the stack of enclosing category ids, outermost first; it is pushed
/// and popped as the walk descends and returns, so each activity records exactly
/// the categories above it. Pre-order, so both output lists come out in render
/// order and no sorting is needed — position in the tree is the order.
fn flatten_items<D: PaneData>(
    items: &[ActivityNode<D>],
    path: &mut Vec<CategoryId>,
    cats: &mut Vec<CategoryMeta>,
    acts: &mut Vec<ActivityWithCategory<D>>,
) {
    for item in items {
        match item {
            ActivityNode::Activity(def) => acts.push(ActivityWithCategory {
                def: def.clone(),
                category: path.last().cloned(),
                path: path.clone(),
            }),
            ActivityNode::Category(cat) => {
                cats.push(CategoryMeta {
                    id: cat.id.clone(),
                    name: cat.name.clone(),
                    icon: cat.icon.clone(),
                    color: cat.color.clone(),
                });
                path.push(cat.id.clone());
                flatten_items(&cat.children, path, cats, acts);
                path.pop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity::{ActivityDef, Category};
    use serde::{Deserialize, Serialize};

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    struct D {
        show_extra: bool,
    }

    fn act(id: &str) -> ActivityDef<D> {
        ActivityDef::new(
            ActivityId::new(id),
            id,
            ActivityIcon::Class(String::new()),
            |_| true,
            |_, _| ().into_any(),
        )
    }

    fn cat(id: &str, children: Vec<ActivityNode<D>>) -> ActivityNode<D> {
        ActivityNode::Category(Category {
            id: CategoryId::new(id),
            name: id.into(),
            icon: ActivityIcon::Class(String::new()),
            color: format!("#{id}"),
            children,
        })
    }

    /// Explorer[ files, Deep[ nested ] ], settings
    fn sample() -> Vec<ActivityNode<D>> {
        vec![
            cat(
                "explorer",
                vec![
                    ActivityNode::activity(act("files")),
                    cat("deep", vec![ActivityNode::activity(act("nested"))]),
                ],
            ),
            ActivityNode::activity(act("settings")),
        ]
    }

    fn flatten(items: &[ActivityNode<D>]) -> (Vec<CategoryMeta>, Vec<ActivityWithCategory<D>>) {
        let (mut cats, mut acts) = (Vec::new(), Vec::new());
        flatten_items(items, &mut Vec::new(), &mut cats, &mut acts);
        (cats, acts)
    }

    #[test]
    fn flatten_records_the_full_ancestor_path() {
        let (_, acts) = flatten(&sample());
        let find = |id: &str| {
            acts.iter()
                .find(|a| a.def.id == ActivityId::new(id))
                .expect("activity present")
        };

        // Top-level activity: no ancestors at all.
        assert!(find("settings").path.is_empty());
        assert_eq!(find("settings").category, None);

        // One level down.
        assert_eq!(find("files").path, vec![CategoryId::new("explorer")]);

        // Two levels down: the whole chain, outermost first, with `category`
        // being the *nearest* ancestor — the one whose colour it inherits.
        assert_eq!(
            find("nested").path,
            vec![CategoryId::new("explorer"), CategoryId::new("deep")]
        );
        assert_eq!(find("nested").category, Some(CategoryId::new("deep")));
    }

    #[test]
    fn flatten_preserves_registration_order() {
        // Position is the order — there is no `order` field to sort by, so a
        // pre-order walk must come out in the order the host wrote.
        let (cats, acts) = flatten(&sample());
        assert_eq!(
            cats.iter().map(|c| c.id.0.as_str()).collect::<Vec<_>>(),
            vec!["explorer", "deep"]
        );
        assert_eq!(
            acts.iter().map(|a| a.def.id.0.as_str()).collect::<Vec<_>>(),
            vec!["files", "nested", "settings"],
            "settings is registered last and must stay last"
        );
    }

    #[test]
    fn path_stack_does_not_leak_between_siblings() {
        // Regression guard on the push/pop: a category's ancestors must not
        // bleed onto its later siblings.
        let items = vec![
            cat("first", vec![ActivityNode::activity(act("inside"))]),
            ActivityNode::activity(act("after")),
            cat(
                "second",
                vec![ActivityNode::activity(act("deep-in-second"))],
            ),
        ];
        let (_, acts) = flatten(&items);
        let path = |id: &str| {
            acts.iter()
                .find(|a| a.def.id == ActivityId::new(id))
                .map(|a| a.path.clone())
                .unwrap()
        };
        assert_eq!(path("inside"), vec![CategoryId::new("first")]);
        assert!(
            path("after").is_empty(),
            "sibling after a category is top-level"
        );
        assert_eq!(path("deep-in-second"), vec![CategoryId::new("second")]);
    }

    #[test]
    fn both_groups_flatten_into_the_lookup_tables() {
        // The bottom group is not decorative: `PaneContent` resolves a pane's
        // active activity through `activities`, so an entry only reachable from
        // `bottom_items` would render as "activity not found" if it were skipped.
        let top = vec![cat("explorer", vec![ActivityNode::activity(act("files"))])];
        let bottom = vec![
            ActivityNode::activity(act("settings")),
            cat("admin", vec![ActivityNode::activity(act("users"))]),
        ];
        let (cats, acts) = flatten_groups(&top, &bottom);

        assert_eq!(
            acts.iter().map(|a| a.def.id.0.as_str()).collect::<Vec<_>>(),
            vec!["files", "settings", "users"],
            "top group first, so flattened order matches the order down the bar"
        );
        assert_eq!(
            cats.iter().map(|c| c.id.0.as_str()).collect::<Vec<_>>(),
            vec!["explorer", "admin"]
        );
        // Nesting works the same in either group.
        let users = acts
            .iter()
            .find(|a| a.def.id == ActivityId::new("users"))
            .unwrap();
        assert_eq!(users.path, vec![CategoryId::new("admin")]);
        // And a bare bottom entry is still top-level, not parented to anything.
        let settings = acts
            .iter()
            .find(|a| a.def.id == ActivityId::new("settings"))
            .unwrap();
        assert!(settings.path.is_empty());
    }

    #[test]
    #[should_panic(expected = "activity id registered more than once")]
    fn duplicate_activity_id_across_groups_is_caught() {
        // The copy-paste case, and the one two groups make easy: neither list
        // looks wrong on its own. Without the guard this renders two bar entries
        // that select each other, and the pane resolves whichever came first.
        let top = vec![ActivityNode::activity(act("settings"))];
        let bottom = vec![ActivityNode::activity(act("settings"))];
        flatten_groups(&top, &bottom);
    }

    #[test]
    #[should_panic(expected = "activity id registered more than once")]
    fn duplicate_activity_id_within_a_group_is_caught() {
        let top = vec![
            cat("explorer", vec![ActivityNode::activity(act("files"))]),
            ActivityNode::activity(act("files")),
        ];
        flatten_groups(&top, &[]);
    }

    #[test]
    #[should_panic(expected = "category id registered more than once")]
    fn duplicate_category_id_is_caught() {
        // Expansion state is keyed by CategoryId, so duplicates would open and
        // close in lockstep.
        let top = vec![
            cat("tools", vec![ActivityNode::activity(act("a"))]),
            cat("tools", vec![ActivityNode::activity(act("b"))]),
        ];
        flatten_groups(&top, &[]);
    }

    #[test]
    fn distinct_ids_across_groups_are_fine() {
        // The guard must not fire on the ordinary case — an activity and a
        // category may share a *name*, and ids are namespaced separately.
        let top = vec![cat("settings", vec![ActivityNode::activity(act("themes"))])];
        let bottom = vec![ActivityNode::activity(act("settings-page"))];
        let (cats, acts) = flatten_groups(&top, &bottom);
        assert_eq!(cats.len(), 1);
        assert_eq!(acts.len(), 2);
    }
}
