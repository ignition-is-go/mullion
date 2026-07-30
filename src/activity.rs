use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::tree::{ActivityId, CategoryId, PaneData, PaneId};

/// One entry in the activity bar, at any depth.
///
/// The bar is registered as an ordered list of these, and a category holds
/// another list of the same — so an activity can sit at the top level beside
/// categories, or nested arbitrarily deep inside them.
///
/// **Position is the order.** There is no `order` field anywhere in this tree;
/// entries render in the order given. To put a settings activity last, put it
/// last.
///
/// ```ignore
/// use mullion::{ActivityNode, Category, ActivityDef};
///
/// let items = vec![
///     ActivityNode::Category(Category {
///         id: CategoryId::new("explorer"),
///         name: "Explorer".into(),
///         icon: folder_icon(),
///         color: "#75beff".into(),
///         children: vec![
///             ActivityNode::activity(files),
///             ActivityNode::activity(outline),
///         ],
///     }),
///     // A bare activity at the top level — no category wrapper, no
///     // expand-to-reveal-one-child step.
///     ActivityNode::activity(settings),
/// ];
/// ```
pub enum ActivityNode<D: PaneData> {
    Activity(ActivityDef<D>),
    Category(Category<D>),
}

impl<D: PaneData> ActivityNode<D> {
    /// Wrap an activity as a node. Shorter than `ActivityNode::Activity(..)` at
    /// the call site, where these nest several deep.
    pub fn activity(def: ActivityDef<D>) -> Self {
        ActivityNode::Activity(def)
    }

    /// Wrap a category as a node.
    pub fn category(cat: Category<D>) -> Self {
        ActivityNode::Category(cat)
    }
}

impl<D: PaneData> Clone for ActivityNode<D> {
    fn clone(&self) -> Self {
        match self {
            ActivityNode::Activity(a) => ActivityNode::Activity(a.clone()),
            ActivityNode::Category(c) => ActivityNode::Category(c.clone()),
        }
    }
}

/// A category of activities. Holds [`ActivityNode`]s, so it can contain
/// activities, further categories, or a mix.
pub struct Category<D: PaneData> {
    pub id: CategoryId,
    pub name: String,
    /// Icon for the category header.
    pub icon: ActivityIcon,
    /// Color for the category. Used for its active indicator and border, and
    /// inherited by descendant activities that have no nearer category.
    pub color: String,
    /// Entries in this category, in render order.
    pub children: Vec<ActivityNode<D>>,
}

impl<D: PaneData> Clone for Category<D> {
    fn clone(&self) -> Self {
        Category {
            id: self.id.clone(),
            name: self.name.clone(),
            icon: self.icon.clone(),
            color: self.color.clone(),
            children: self.children.clone(),
        }
    }
}

/// Serializable category metadata (without children, for internal use).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CategoryMeta {
    pub id: CategoryId,
    pub name: String,
    pub icon: ActivityIcon,
    pub color: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ActivityIcon {
    /// CSS class name (e.g. for icon fonts).
    Class(String),
    /// Inline SVG markup.
    Svg(String),
    /// URL to an image.
    Url(String),
}

/// Renders content for a specific pane, given its id and a reactive slice of
/// its data. Used for both an activity's body ([`ActivityDef::render`]) and its
/// optional header content ([`ActivityDef::header`]).
pub type ActivityRender<D> = fn(PaneId, Signal<D>) -> AnyView;

/// Definition of an activity, registered at startup by the consuming app.
///
/// Construct with a struct literal, or use [`ActivityDef::new`] +
/// [`ActivityDef::with_header`] — the builder form is forward-compatible with
/// future optional fields, so it won't break when this struct grows.
pub struct ActivityDef<D: PaneData> {
    pub id: ActivityId,
    pub name: String,
    pub icon: ActivityIcon,
    /// Return true if this activity should appear in a pane with the given data.
    pub filter: fn(&D) -> bool,
    /// Render this activity's content for a specific pane.
    ///
    /// The `Signal<D>` fires only when *this* pane's data changes — not
    /// when other panes update or when the tree structure shifts.
    pub render: ActivityRender<D>,
    /// Optional custom content rendered in the pane header band, beside the
    /// activity's name. Receives the same `(PaneId, Signal<D>)` as `render`,
    /// so it can react to this pane's data (e.g. show the current scene's
    /// name). `None` ⇒ the header band shows just the activity name.
    pub header: Option<ActivityRender<D>>,
}

impl<D: PaneData> ActivityDef<D> {
    /// Create an activity with no custom header content (the header band shows
    /// just `name`). Add custom content with [`ActivityDef::with_header`].
    pub fn new(
        id: ActivityId,
        name: impl Into<String>,
        icon: ActivityIcon,
        filter: fn(&D) -> bool,
        render: ActivityRender<D>,
    ) -> Self {
        ActivityDef {
            id,
            name: name.into(),
            icon,
            filter,
            render,
            header: None,
        }
    }

    /// Set custom header content rendered beside the activity name.
    pub fn with_header(mut self, header: ActivityRender<D>) -> Self {
        self.header = Some(header);
        self
    }
}

impl<D: PaneData> Clone for ActivityDef<D> {
    fn clone(&self) -> Self {
        ActivityDef {
            id: self.id.clone(),
            name: self.name.clone(),
            icon: self.icon.clone(),
            filter: self.filter,
            render: self.render,
            header: self.header,
        }
    }
}

/// Internal representation pairing an activity with where it sits in the tree.
///
/// `path` is its ancestor categories, outermost first — empty for a top-level
/// activity. `category` is the nearest ancestor (the last element of `path`),
/// which is the one whose colour the activity inherits.
pub struct ActivityWithCategory<D: PaneData> {
    pub def: ActivityDef<D>,
    pub category: Option<CategoryId>,
    pub path: Vec<CategoryId>,
}

impl<D: PaneData> Clone for ActivityWithCategory<D> {
    fn clone(&self) -> Self {
        ActivityWithCategory {
            def: self.def.clone(),
            category: self.category.clone(),
            path: self.path.clone(),
        }
    }
}
