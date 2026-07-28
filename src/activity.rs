use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::tree::{ActivityId, CategoryId, PaneData, PaneId};

/// A category of activities, containing its activity definitions.
pub struct Category<D: PaneData> {
    pub id: CategoryId,
    pub name: String,
    pub order: u32,
    /// Icon for the category header.
    pub icon: ActivityIcon,
    /// Color for the category (used for active indicators, borders, etc.).
    pub color: String,
    /// Activities in this category.
    pub activities: Vec<ActivityDef<D>>,
}

impl<D: PaneData> Clone for Category<D> {
    fn clone(&self) -> Self {
        Category {
            id: self.id.clone(),
            name: self.name.clone(),
            order: self.order,
            icon: self.icon.clone(),
            color: self.color.clone(),
            activities: self.activities.clone(),
        }
    }
}

/// Serializable category metadata (without activities, for internal use).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CategoryMeta {
    pub id: CategoryId,
    pub name: String,
    pub order: u32,
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

/// Internal representation pairing an activity with its category id. `category`
/// is `None` for a free-floating activity (registered outside any category —
/// rendered as a top-level icon in the activity bar).
pub struct ActivityWithCategory<D: PaneData> {
    pub def: ActivityDef<D>,
    pub category: Option<CategoryId>,
}

impl<D: PaneData> Clone for ActivityWithCategory<D> {
    fn clone(&self) -> Self {
        ActivityWithCategory {
            def: self.def.clone(),
            category: self.category.clone(),
        }
    }
}
