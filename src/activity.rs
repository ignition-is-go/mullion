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
            id: self.id,
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

/// Definition of an activity, registered at startup by the consuming app.
pub struct ActivityDef<D: PaneData> {
    pub id: ActivityId,
    pub name: String,
    pub icon: ActivityIcon,
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
            filter: self.filter,
            render: self.render,
        }
    }
}

/// Internal representation pairing an activity with its category id.
pub struct ActivityWithCategory<D: PaneData> {
    pub def: ActivityDef<D>,
    pub category: CategoryId,
}

impl<D: PaneData> Clone for ActivityWithCategory<D> {
    fn clone(&self) -> Self {
        ActivityWithCategory {
            def: self.def.clone(),
            category: self.category,
        }
    }
}
