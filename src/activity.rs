use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::tree::{ActivityId, CategoryId, PaneData, PaneId};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Category {
    pub id: CategoryId,
    pub name: String,
    pub order: u32,
    /// Icon for the category header.
    pub icon: ActivityIcon,
    /// Color for the category (used for active indicators, borders, etc.).
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

/// Definition of an activity, registered at startup by the consuming app.
///
/// Generic over `D` — the consumer's pane data type. The `filter` function
/// determines whether this activity appears in a given pane, and `render`
/// produces the view for the activity's content area.
pub struct ActivityDef<D: PaneData> {
    pub id: ActivityId,
    pub name: String,
    pub icon: ActivityIcon,
    pub category: CategoryId,
    /// Return true if this activity should appear in a pane with the given data.
    pub filter: fn(&D) -> bool,
    /// Render this activity's content for a specific pane.
    pub render: fn(PaneId, ReadSignal<D>) -> AnyView,
}

impl<D: PaneData> Clone for ActivityDef<D> {
    fn clone(&self) -> Self {
        ActivityDef {
            id: self.id,
            name: self.name.clone(),
            icon: self.icon.clone(),
            category: self.category,
            filter: self.filter,
            render: self.render,
        }
    }
}
