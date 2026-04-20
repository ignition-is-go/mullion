use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct PaneId(pub String);

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct ActivityId(pub String);

#[derive(Clone, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct CategoryId(pub String);

impl PaneId {
    pub fn new(id: impl Into<String>) -> Self {
        PaneId(id.into())
    }
}

impl ActivityId {
    pub fn new(id: impl Into<String>) -> Self {
        ActivityId(id.into())
    }
}

impl CategoryId {
    pub fn new(id: impl Into<String>) -> Self {
        CategoryId(id.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DropEdge {
    Top,
    Bottom,
    Left,
    Right,
    Center,
}

impl DropEdge {
    pub fn split_direction(&self) -> SplitDirection {
        match self {
            DropEdge::Top | DropEdge::Bottom => SplitDirection::Vertical,
            DropEdge::Left | DropEdge::Right | DropEdge::Center => SplitDirection::Horizontal,
        }
    }

    pub fn source_is_first(&self) -> bool {
        matches!(self, DropEdge::Top | DropEdge::Left)
    }
}

/// Trait bound alias for consumer-defined pane data.
///
/// `Send + Sync` is required because per-leaf reactive slices are stored in
/// Leptos `Signal<D>` (which uses `SyncStorage`). Nearly all consumer data
/// types already satisfy this — any plain data struct without thread-hostile
/// contents (raw pointers, `Rc`, etc.) will compile.
pub trait PaneData:
    Clone + PartialEq + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static
{
}

impl<T> PaneData for T where
    T: Clone + PartialEq + Serialize + for<'de> Deserialize<'de> + Send + Sync + 'static
{
}

/// A node in the pane tree — either a leaf pane or a split containing two children.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound = "")]
pub enum PaneNode<D: PaneData> {
    Leaf {
        id: PaneId,
        active_activity: Option<ActivityId>,
        data: D,
    },
    Split {
        direction: SplitDirection,
        /// Fraction (0.0..1.0) of space allocated to `first`.
        ratio: f64,
        first: Box<PaneNode<D>>,
        second: Box<PaneNode<D>>,
    },
}

impl<D: PaneData> PaneNode<D> {
    /// Create a new leaf node.
    pub fn leaf(id: PaneId, data: D) -> Self {
        PaneNode::Leaf {
            id,
            active_activity: None,
            data,
        }
    }

    /// Create a leaf with an initial active activity.
    pub fn leaf_with_activity(id: PaneId, activity: ActivityId, data: D) -> Self {
        PaneNode::Leaf {
            id,
            active_activity: Some(activity),
            data,
        }
    }

    /// Find a leaf by id (immutable).
    pub fn find(&self, target: &PaneId) -> Option<&PaneNode<D>> {
        match self {
            PaneNode::Leaf { id, .. } if id == target => Some(self),
            PaneNode::Split { first, second, .. } => {
                first.find(target).or_else(|| second.find(target))
            }
            _ => None,
        }
    }

    /// Find a leaf by id (mutable).
    pub fn find_mut(&mut self, target: &PaneId) -> Option<&mut PaneNode<D>> {
        match self {
            PaneNode::Leaf { id, .. } if id == target => Some(self),
            PaneNode::Split { first, second, .. } => {
                first.find_mut(target).or_else(|| second.find_mut(target))
            }
            _ => None,
        }
    }

    /// Split a leaf pane. The original becomes `first`, new pane becomes `second`.
    pub fn split(
        &mut self,
        target: &PaneId,
        direction: SplitDirection,
        new_id: PaneId,
        new_data: D,
    ) -> bool {
        if let PaneNode::Leaf { id, active_activity, .. } = self {
            if id == target {
                let inherit_activity = active_activity.clone();
                let original = std::mem::replace(self, PaneNode::leaf(PaneId::new("__temp__"), new_data.clone()));
                let new_leaf = PaneNode::Leaf {
                    id: new_id,
                    active_activity: inherit_activity,
                    data: new_data,
                };
                *self = PaneNode::Split {
                    direction,
                    ratio: 0.5,
                    first: Box::new(original),
                    second: Box::new(new_leaf),
                };
                return true;
            }
        }
        if let PaneNode::Split { first, second, .. } = self {
            if first.split(target, direction, new_id.clone(), new_data.clone()) {
                return true;
            }
            return second.split(target, direction, new_id, new_data);
        }
        false
    }

    /// Remove a pane, collapsing its parent split.
    pub fn close(&mut self, target: &PaneId) -> Option<D> {
        self.close_inner(target).map(|(data, _)| data)
    }

    fn close_inner(&mut self, target: &PaneId) -> Option<(D, bool)> {
        match self {
            PaneNode::Leaf { id, .. } if id == target => None,
            PaneNode::Split { first, second, .. } => {
                if let PaneNode::Leaf { id, data, .. } = first.as_ref() {
                    if id == target {
                        let data = data.clone();
                        let sibling = *second.clone();
                        *self = sibling;
                        return Some((data, true));
                    }
                }
                if let PaneNode::Leaf { id, data, .. } = second.as_ref() {
                    if id == target {
                        let data = data.clone();
                        let sibling = *first.clone();
                        *self = sibling;
                        return Some((data, true));
                    }
                }
                if let Some(result) = first.close_inner(target) {
                    return Some(result);
                }
                second.close_inner(target)
            }
            _ => None,
        }
    }

    /// Change the split direction of the immediate parent of `target`.
    pub fn change_direction(&mut self, target: &PaneId, new_direction: SplitDirection) -> bool {
        match self {
            PaneNode::Split { direction, first, second, .. } => {
                let first_contains = first.contains(target);
                let second_contains = second.contains(target);
                if first_contains || second_contains {
                    let is_direct_child = match (first.as_ref(), second.as_ref()) {
                        (PaneNode::Leaf { id, .. }, _) if id == target => true,
                        (_, PaneNode::Leaf { id, .. }) if id == target => true,
                        _ => false,
                    };
                    if is_direct_child {
                        *direction = new_direction;
                        return true;
                    }
                    if first_contains {
                        return first.change_direction(target, new_direction);
                    }
                    return second.change_direction(target, new_direction);
                }
                false
            }
            _ => false,
        }
    }

    /// Set the ratio of the split identified by `split_key` — the first leaf
    /// id under the split's `second` subtree.
    ///
    /// Keying splits by "first leaf of second" (rather than any leaf under
    /// `first`) is what makes splits addressable without collisions: every
    /// leaf lives in exactly one place in the tree, so it can be the
    /// "leftmost of second" for at most one ancestor split. Using "leftmost
    /// of first" collides the moment you split a pane in place, because the
    /// original pane stays leftmost of both the new outer and inner splits.
    ///
    /// Returns `true` if a matching split was found and updated. Non-finite
    /// ratios (`NaN`, `±inf`) are rejected.
    pub fn set_split_ratio(&mut self, split_key: &PaneId, new_ratio: f64) -> bool {
        if !new_ratio.is_finite() {
            return false;
        }
        let clamped = new_ratio.clamp(0.1, 0.9);
        match self {
            PaneNode::Split { ratio, first, second, .. } => {
                if second.leftmost_leaf_id() == split_key {
                    *ratio = clamped;
                    return true;
                }
                first.set_split_ratio(split_key, new_ratio)
                    || second.set_split_ratio(split_key, new_ratio)
            }
            _ => false,
        }
    }

    /// Move a pane from one position to another.
    pub fn move_pane(&mut self, source: &PaneId, destination: &PaneId, edge: DropEdge) -> bool {
        if source == destination {
            return false;
        }
        let source_leaf = match self.find(source) {
            Some(PaneNode::Leaf { id, data, active_activity, .. }) => {
                (id.clone(), data.clone(), active_activity.clone())
            }
            _ => return false,
        };
        if self.close(source).is_none() {
            return false;
        }
        if let Some(dest_node) = self.find_mut(destination) {
            let direction = edge.split_direction();
            let original = std::mem::replace(dest_node, PaneNode::leaf(PaneId::new("__temp__"), source_leaf.1.clone()));
            let new_leaf = PaneNode::Leaf {
                id: source_leaf.0,
                active_activity: source_leaf.2,
                data: source_leaf.1,
            };
            let (first, second) = if edge.source_is_first() {
                (Box::new(new_leaf), Box::new(original))
            } else {
                (Box::new(original), Box::new(new_leaf))
            };
            *dest_node = PaneNode::Split {
                direction,
                ratio: 0.5,
                first,
                second,
            };
            true
        } else {
            false
        }
    }

    /// Check if this subtree contains a pane with the given id.
    pub fn contains(&self, target: &PaneId) -> bool {
        self.find(target).is_some()
    }

    /// Collect all leaf PaneIds.
    pub fn leaf_ids(&self) -> Vec<PaneId> {
        let mut ids = Vec::new();
        self.collect_ids(&mut ids);
        ids
    }

    fn collect_ids(&self, ids: &mut Vec<PaneId>) {
        match self {
            PaneNode::Leaf { id, .. } => ids.push(id.clone()),
            PaneNode::Split { first, second, .. } => {
                first.collect_ids(ids);
                second.collect_ids(ids);
            }
        }
    }

    /// Returns a reference to the id of the leftmost leaf under this node.
    ///
    /// O(depth), unlike `leaf_ids().into_iter().next()` which is O(subtree
    /// size) because it allocates the full leaf-id vector. Used heavily by
    /// the renderer's rect walks — those fire once per leaf per structural
    /// mutation, so using the O(subtree-size) form there turns a single
    /// structural change into O(N³) work.
    pub(crate) fn leftmost_leaf_id(&self) -> &PaneId {
        let mut node = self;
        loop {
            match node {
                PaneNode::Leaf { id, .. } => return id,
                PaneNode::Split { first, .. } => node = first,
            }
        }
    }
}

/// Look up the ratio of the split whose `split_key` (first-leaf-of-second) equals `key`.
pub(crate) fn find_ratio<D: PaneData>(node: &PaneNode<D>, key: &PaneId) -> Option<f64> {
    match node {
        PaneNode::Leaf { .. } => None,
        PaneNode::Split {
            ratio,
            first,
            second,
            ..
        } => {
            if second.leftmost_leaf_id() == key {
                Some(*ratio)
            } else {
                find_ratio(first, key).or_else(|| find_ratio(second, key))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
    struct D(u32);

    fn sample() -> PaneNode<D> {
        PaneNode::Split {
            direction: SplitDirection::Horizontal,
            ratio: 0.5,
            first: Box::new(PaneNode::leaf(PaneId::new("a"), D(1))),
            second: Box::new(PaneNode::leaf(PaneId::new("b"), D(2))),
        }
    }

    // These tests verify the *structural* view that the renderer subscribes
    // to (`leaf_ids()` and `collect_split_keys()`) is stable across
    // non-structural mutations. The per-leaf render path subscribes only to
    // this structural view plus per-leaf/per-split signals, so any
    // non-structural mutation that DIDN'T change this view also cannot force
    // a re-mount of existing leaves.

    #[test]
    fn leaf_ids_and_split_keys_are_stable_across_ratio_changes() {
        let mut t = sample();
        let ids_before = t.leaf_ids();
        let keys_before = collect_split_keys(&t);
        // `sample()` is Split { first: leaf("a"), second: leaf("b") }.
        // The split's key is "b" (first leaf of second subtree).
        assert!(t.set_split_ratio(&PaneId::new("b"), 0.8));
        assert_eq!(t.leaf_ids(), ids_before);
        assert_eq!(collect_split_keys(&t), keys_before);
    }

    #[test]
    fn leaf_ids_and_split_keys_are_stable_across_data_changes() {
        let mut t = sample();
        let ids_before = t.leaf_ids();
        let keys_before = collect_split_keys(&t);
        if let Some(PaneNode::Leaf { data, .. }) = t.find_mut(&PaneId::new("a")) {
            *data = D(99);
        }
        assert_eq!(t.leaf_ids(), ids_before);
        assert_eq!(collect_split_keys(&t), keys_before);
    }

    #[test]
    fn leaf_ids_and_split_keys_are_stable_across_active_activity_changes() {
        let mut t = sample();
        let ids_before = t.leaf_ids();
        let keys_before = collect_split_keys(&t);
        if let Some(PaneNode::Leaf { active_activity, .. }) = t.find_mut(&PaneId::new("a")) {
            *active_activity = Some(ActivityId::new("foo"));
        }
        assert_eq!(t.leaf_ids(), ids_before);
        assert_eq!(collect_split_keys(&t), keys_before);
    }

    #[test]
    fn split_keys_are_stable_across_direction_changes() {
        // `change_direction` changes a split's direction but not its set of
        // leaves, so its `split_key` (first leaf of `second`) must stay the
        // same — the renderer depends on this to reuse the existing handle
        // instance rather than remounting it.
        let mut t = PaneNode::Split {
            direction: SplitDirection::Horizontal,
            ratio: 0.5,
            first: Box::new(PaneNode::leaf(PaneId::new("a"), D(1))),
            second: Box::new(PaneNode::leaf(PaneId::new("b"), D(2))),
        };
        let keys_before = collect_split_keys(&t);
        assert_eq!(
            find_split_direction(&t, &PaneId::new("b")),
            Some(SplitDirection::Horizontal),
        );
        assert!(t.change_direction(&PaneId::new("a"), SplitDirection::Vertical));
        assert_eq!(collect_split_keys(&t), keys_before);
        assert_eq!(
            find_split_direction(&t, &PaneId::new("b")),
            Some(SplitDirection::Vertical),
        );
    }

    #[test]
    fn leaf_ids_change_on_split() {
        let mut t = sample();
        let before = t.leaf_ids();
        t.split(
            &PaneId::new("a"),
            SplitDirection::Vertical,
            PaneId::new("c"),
            D(3),
        );
        let after = t.leaf_ids();
        assert_ne!(before, after);
        assert!(after.contains(&PaneId::new("c")));
    }

    #[test]
    fn leaf_ids_change_on_close() {
        let mut t = sample();
        t.close(&PaneId::new("a"));
        let after = t.leaf_ids();
        assert!(!after.contains(&PaneId::new("a")));
    }

    #[test]
    fn collect_split_ratios_walks_nested_splits() {
        // Key each split by the first leaf of its `second` subtree — that
        // value is unique across nested splits (unlike `first`'s leftmost
        // leaf, which collides when splitting a pane in place).
        let t = PaneNode::Split {
            direction: SplitDirection::Horizontal,
            ratio: 0.3,
            first: Box::new(PaneNode::leaf(PaneId::new("a"), D(1))),
            second: Box::new(PaneNode::Split {
                direction: SplitDirection::Vertical,
                ratio: 0.7,
                first: Box::new(PaneNode::leaf(PaneId::new("b"), D(2))),
                second: Box::new(PaneNode::leaf(PaneId::new("c"), D(3))),
            }),
        };
        let mut out = Vec::new();
        collect_split_ratios(&t, &mut out);
        assert_eq!(out.len(), 2);
        // Outer: second is the inner split; inner's first leaf is "b".
        assert_eq!(out[0], (PaneId::new("b"), 0.3));
        // Inner: second is leaf("c").
        assert_eq!(out[1], (PaneId::new("c"), 0.7));
    }

    #[test]
    fn split_keys_are_unique_when_splitting_in_place() {
        // Regression: the previous keying (first-leaf-of-first) collided
        // when a pane was split in place, because the original pane stayed
        // leftmost of both the outer and inner splits.
        let mut t = PaneNode::Split {
            direction: SplitDirection::Horizontal,
            ratio: 0.25,
            first: Box::new(PaneNode::leaf(PaneId::new("sidebar"), D(1))),
            second: Box::new(PaneNode::leaf(PaneId::new("main"), D(2))),
        };
        t.split(
            &PaneId::new("sidebar"),
            SplitDirection::Horizontal,
            PaneId::new("new"),
            D(3),
        );

        let mut keys = Vec::new();
        collect_split_ratios(&t, &mut keys);
        let key_set: std::collections::HashSet<_> = keys.iter().map(|(k, _)| k.clone()).collect();
        assert_eq!(
            key_set.len(),
            keys.len(),
            "split keys must be unique, got {:?}",
            keys,
        );
    }

    #[test]
    fn rect_split_horizontal_preserves_total_width() {
        let r = Rect::FULL;
        let (a, b) = r.split(SplitDirection::Horizontal, 0.3);
        assert!((a.width + b.width - r.width).abs() < 1e-9);
        assert_eq!(a.height, r.height);
        assert_eq!(b.height, r.height);
        assert_eq!(b.left, r.left + a.width);
        assert!((a.width - 0.3).abs() < 1e-9);
    }

    #[test]
    fn rect_split_vertical_preserves_total_height() {
        let r = Rect::FULL;
        let (a, b) = r.split(SplitDirection::Vertical, 0.75);
        assert!((a.height + b.height - r.height).abs() < 1e-9);
        assert_eq!(a.width, r.width);
        assert_eq!(b.width, r.width);
        assert_eq!(b.top, r.top + a.height);
        assert!((a.height - 0.75).abs() < 1e-9);
    }

    #[test]
    fn leaf_rect_walks_nested_splits() {
        // sidebar(25%) | main-top(60% of right) / main-bottom(40% of right)
        let t = PaneNode::Split {
            direction: SplitDirection::Horizontal,
            ratio: 0.25,
            first: Box::new(PaneNode::leaf(PaneId::new("sidebar"), D(1))),
            second: Box::new(PaneNode::Split {
                direction: SplitDirection::Vertical,
                ratio: 0.6,
                first: Box::new(PaneNode::leaf(PaneId::new("main-top"), D(2))),
                second: Box::new(PaneNode::leaf(PaneId::new("main-bottom"), D(3))),
            }),
        };
        let read = |_: &PaneId| 0.25; // unused because we'll use tree ratios
        // Actually use the tree's stored ratios via a closure
        let read_from_tree = |k: &PaneId| {
            let mut out = Vec::new();
            collect_split_ratios(&t, &mut out);
            out.iter().find(|(key, _)| key == k).map(|(_, r)| *r).unwrap_or(0.5)
        };

        let sidebar_rect = leaf_rect(&t, &PaneId::new("sidebar"), read_from_tree).unwrap();
        assert_eq!(sidebar_rect, Rect { left: 0.0, top: 0.0, width: 0.25, height: 1.0 });

        let top_rect = leaf_rect(&t, &PaneId::new("main-top"), read_from_tree).unwrap();
        assert!((top_rect.left - 0.25).abs() < 1e-9);
        assert_eq!(top_rect.top, 0.0);
        assert!((top_rect.width - 0.75).abs() < 1e-9);
        assert!((top_rect.height - 0.6).abs() < 1e-9);

        let bot_rect = leaf_rect(&t, &PaneId::new("main-bottom"), read_from_tree).unwrap();
        assert!((bot_rect.left - 0.25).abs() < 1e-9);
        assert!((bot_rect.top - 0.6).abs() < 1e-9);
        assert!((bot_rect.width - 0.75).abs() < 1e-9);
        assert!((bot_rect.height - 0.4).abs() < 1e-9);

        let _ = read; // silence unused
    }

    #[test]
    fn split_parent_rect_returns_enclosing_area() {
        let t = PaneNode::Split {
            direction: SplitDirection::Horizontal,
            ratio: 0.25,
            first: Box::new(PaneNode::leaf(PaneId::new("sidebar"), D(1))),
            second: Box::new(PaneNode::Split {
                direction: SplitDirection::Vertical,
                ratio: 0.6,
                first: Box::new(PaneNode::leaf(PaneId::new("main-top"), D(2))),
                second: Box::new(PaneNode::leaf(PaneId::new("main-bottom"), D(3))),
            }),
        };
        let read_from_tree = |k: &PaneId| {
            let mut out = Vec::new();
            collect_split_ratios(&t, &mut out);
            out.iter().find(|(key, _)| key == k).map(|(_, r)| *r).unwrap_or(0.5)
        };

        // Outer split is at the root — parent rect is FULL.
        let outer = split_parent_rect(&t, &PaneId::new("main-top"), read_from_tree).unwrap();
        assert_eq!(outer, Rect::FULL);

        // Inner split occupies the right 75%.
        let inner = split_parent_rect(&t, &PaneId::new("main-bottom"), read_from_tree).unwrap();
        assert!((inner.left - 0.25).abs() < 1e-9);
        assert_eq!(inner.top, 0.0);
        assert!((inner.width - 0.75).abs() < 1e-9);
        assert_eq!(inner.height, 1.0);
    }

    #[test]
    fn find_ratio_returns_correct_value() {
        let t = PaneNode::Split {
            direction: SplitDirection::Horizontal,
            ratio: 0.42,
            first: Box::new(PaneNode::leaf(PaneId::new("a"), D(1))),
            second: Box::new(PaneNode::leaf(PaneId::new("b"), D(2))),
        };
        // Keyed by first-leaf-of-second: "b".
        assert_eq!(find_ratio(&t, &PaneId::new("b")), Some(0.42));
        assert_eq!(find_ratio(&t, &PaneId::new("a")), None);
        assert_eq!(find_ratio(&t, &PaneId::new("nonexistent")), None);
    }

    #[test]
    fn set_split_ratio_rejects_non_finite() {
        let mut t = sample();
        assert!(!t.set_split_ratio(&PaneId::new("b"), f64::NAN));
        assert!(!t.set_split_ratio(&PaneId::new("b"), f64::INFINITY));
        assert!(!t.set_split_ratio(&PaneId::new("b"), f64::NEG_INFINITY));
        // Ratio unchanged
        if let PaneNode::Split { ratio, .. } = &t {
            assert_eq!(*ratio, 0.5);
        } else {
            panic!("sample should be a split");
        }
    }

    #[test]
    fn set_split_ratio_updates_correct_split_when_nested() {
        let mut t = PaneNode::Split {
            direction: SplitDirection::Horizontal,
            ratio: 0.25,
            first: Box::new(PaneNode::leaf(PaneId::new("sidebar"), D(1))),
            second: Box::new(PaneNode::leaf(PaneId::new("main"), D(2))),
        };
        t.split(
            &PaneId::new("sidebar"),
            SplitDirection::Horizontal,
            PaneId::new("new"),
            D(3),
        );
        // After split:
        //   outer = Split{ first=Split{ first=leaf(sidebar), second=leaf(new) },
        //                  second=leaf(main) }
        // Split keys (first-leaf-of-second):
        //   outer.second is leaf(main) → outer key = "main"
        //   inner.second is leaf(new)  → inner key = "new"
        assert!(t.set_split_ratio(&PaneId::new("main"), 0.6));
        assert!(t.set_split_ratio(&PaneId::new("new"), 0.3));

        // A leaf id that isn't any split's key should not match.
        assert!(!t.set_split_ratio(&PaneId::new("sidebar"), 0.9));

        // Verify the right splits got the right ratios.
        let mut collected = Vec::new();
        collect_split_ratios(&t, &mut collected);
        let outer = collected
            .iter()
            .find(|(k, _)| k == &PaneId::new("main"))
            .expect("outer split");
        let inner = collected
            .iter()
            .find(|(k, _)| k == &PaneId::new("new"))
            .expect("inner split");
        assert!((outer.1 - 0.6).abs() < 1e-9);
        assert!((inner.1 - 0.3).abs() < 1e-9);
    }
}

/// An axis-aligned rectangle in unit (0.0..=1.0) coordinates, representing a
/// fractional area of the root pane container. Used internally by the
/// flat-layout renderer to position leaves and split handles.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Rect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

impl Rect {
    /// The rectangle covering the entire container.
    pub(crate) const FULL: Rect = Rect {
        left: 0.0,
        top: 0.0,
        width: 1.0,
        height: 1.0,
    };

    /// Divide this rect by a split at the given direction and ratio. Returns
    /// `(first, second)` — `first` gets `ratio` of the axis that `direction`
    /// splits, `second` gets the remainder.
    pub(crate) fn split(&self, direction: SplitDirection, ratio: f64) -> (Rect, Rect) {
        match direction {
            SplitDirection::Horizontal => {
                let first_w = self.width * ratio;
                let first = Rect {
                    left: self.left,
                    top: self.top,
                    width: first_w,
                    height: self.height,
                };
                let second = Rect {
                    left: self.left + first_w,
                    top: self.top,
                    width: self.width - first_w,
                    height: self.height,
                };
                (first, second)
            }
            SplitDirection::Vertical => {
                let first_h = self.height * ratio;
                let first = Rect {
                    left: self.left,
                    top: self.top,
                    width: self.width,
                    height: first_h,
                };
                let second = Rect {
                    left: self.left,
                    top: self.top + first_h,
                    width: self.width,
                    height: self.height - first_h,
                };
                (first, second)
            }
        }
    }
}

/// Walk the tree to find `target`'s rect, given a function that resolves a
/// split's live ratio by its `split_key`.
///
/// The walk only reads ratios on the leaf's ancestor chain, so when this is
/// called from a `Memo`, only those ratios are tracked as dependencies —
/// resizing an unrelated split will not invalidate the memo.
pub(crate) fn leaf_rect<D: PaneData>(
    tree: &PaneNode<D>,
    target: &PaneId,
    mut read_ratio: impl FnMut(&PaneId) -> f64,
) -> Option<Rect> {
    fn walk<D: PaneData>(
        node: &PaneNode<D>,
        target: &PaneId,
        rect: Rect,
        read_ratio: &mut dyn FnMut(&PaneId) -> f64,
    ) -> Option<Rect> {
        match node {
            PaneNode::Leaf { id, .. } if id == target => Some(rect),
            PaneNode::Leaf { .. } => None,
            PaneNode::Split {
                direction,
                first,
                second,
                ..
            } => {
                let ratio = read_ratio(second.leftmost_leaf_id());
                let (first_rect, second_rect) = rect.split(*direction, ratio);
                walk(first, target, first_rect, read_ratio)
                    .or_else(|| walk(second, target, second_rect, read_ratio))
            }
        }
    }
    walk(tree, target, Rect::FULL, &mut read_ratio)
}

/// Walk the tree to find the parent rect of the split identified by `split_key`
/// (the first leaf id under the split's `second` subtree).
pub(crate) fn split_parent_rect<D: PaneData>(
    tree: &PaneNode<D>,
    split_key: &PaneId,
    mut read_ratio: impl FnMut(&PaneId) -> f64,
) -> Option<Rect> {
    fn walk<D: PaneData>(
        node: &PaneNode<D>,
        split_key: &PaneId,
        rect: Rect,
        read_ratio: &mut dyn FnMut(&PaneId) -> f64,
    ) -> Option<Rect> {
        match node {
            PaneNode::Leaf { .. } => None,
            PaneNode::Split {
                direction,
                first,
                second,
                ..
            } => {
                let this_key = second.leftmost_leaf_id();
                if this_key == split_key {
                    return Some(rect);
                }
                let ratio = read_ratio(this_key);
                let (first_rect, second_rect) = rect.split(*direction, ratio);
                walk(first, split_key, first_rect, read_ratio)
                    .or_else(|| walk(second, split_key, second_rect, read_ratio))
            }
        }
    }
    walk(tree, split_key, Rect::FULL, &mut read_ratio)
}

/// Walk the tree collecting the `split_key` of every split (first leaf id
/// under each split's `second` subtree).
///
/// Used by the renderer's flat layout to enumerate split handles, keyed
/// stably by `split_key` regardless of their direction. Direction is read
/// reactively per-handle via [`find_split_direction`] so a direction change
/// updates the existing handle rather than remounting it.
pub(crate) fn collect_split_keys<D: PaneData>(node: &PaneNode<D>) -> Vec<PaneId> {
    let mut out = Vec::new();
    fn walk<D: PaneData>(node: &PaneNode<D>, out: &mut Vec<PaneId>) {
        if let PaneNode::Split { first, second, .. } = node {
            out.push(second.leftmost_leaf_id().clone());
            walk(first, out);
            walk(second, out);
        }
    }
    walk(node, &mut out);
    out
}

/// Find the direction of the split identified by `split_key`.
pub(crate) fn find_split_direction<D: PaneData>(
    node: &PaneNode<D>,
    split_key: &PaneId,
) -> Option<SplitDirection> {
    match node {
        PaneNode::Leaf { .. } => None,
        PaneNode::Split {
            direction,
            first,
            second,
            ..
        } => {
            if second.leftmost_leaf_id() == split_key {
                Some(*direction)
            } else {
                find_split_direction(first, split_key)
                    .or_else(|| find_split_direction(second, split_key))
            }
        }
    }
}

/// Walk the tree collecting `(split_key, ratio)` for every split, where
/// `split_key` is the first leaf id under the split's `second` subtree.
pub(crate) fn collect_split_ratios<D: PaneData>(
    node: &PaneNode<D>,
    out: &mut Vec<(PaneId, f64)>,
) {
    if let PaneNode::Split {
        ratio,
        first,
        second,
        ..
    } = node
    {
        out.push((second.leftmost_leaf_id().clone(), *ratio));
        collect_split_ratios(first, out);
        collect_split_ratios(second, out);
    }
}
