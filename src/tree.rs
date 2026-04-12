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
pub trait PaneData:
    Clone + PartialEq + Serialize + for<'de> Deserialize<'de> + 'static
{
}

impl<T> PaneData for T where
    T: Clone + PartialEq + Serialize + for<'de> Deserialize<'de> + 'static
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

    /// Set the ratio of the deepest split whose `first` child contains `target`.
    pub fn set_ratio(&mut self, target: &PaneId, new_ratio: f64) -> bool {
        let clamped = new_ratio.clamp(0.1, 0.9);
        match self {
            PaneNode::Split { ratio, first, second, .. } => {
                if first.set_ratio(target, new_ratio) || second.set_ratio(target, new_ratio) {
                    return true;
                }
                if first.contains(target) {
                    *ratio = clamped;
                    return true;
                }
                false
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
}
