/// Theme for the activity bar.
#[derive(Clone, Debug)]
pub struct ActivityBarTheme {
    /// Width of the activity bar when collapsed (e.g. "28px").
    pub width: String,
    /// Width of the activity bar when expanded/hovered (e.g. "150px").
    pub expanded_width: String,
    /// Size of activity icons (e.g. "14px").
    pub icon_size: String,
    /// Background color of the bar.
    pub background: String,
    /// Border (CSS shorthand, e.g. "1px solid #333").
    pub border: String,
    /// Border radius.
    pub border_radius: String,
    /// Padding right when expanded (space after labels).
    pub expanded_padding: String,
    /// Font size for labels.
    pub font_size: String,
    /// Icon fill color.
    pub icon_color: String,
    /// Icon stroke color.
    pub icon_stroke_color: String,
    /// Icon opacity when inactive.
    pub icon_opacity: String,
    /// Icon opacity when active.
    pub icon_active_opacity: String,
    /// Width of the category border on expanded activity lists.
    pub category_border_width: String,
}

impl Default for ActivityBarTheme {
    fn default() -> Self {
        ActivityBarTheme {
            width: "28px".into(),
            expanded_width: "150px".into(),
            icon_size: "14px".into(),
            background: "transparent".into(),
            border: "none".into(),
            border_radius: "0".into(),
            expanded_padding: "8px".into(),
            font_size: "11px".into(),
            icon_color: "currentColor".into(),
            icon_stroke_color: "currentColor".into(),
            icon_opacity: "0.5".into(),
            icon_active_opacity: "1".into(),
            category_border_width: "2px".into(),
        }
    }
}

/// Style for split handles, powered by css-styled.
///
/// Generates scoped CSS with typed class names, CSS custom properties,
/// and spec-validated base CSS.
#[derive(css_styled::StyledComponent, Clone, Debug)]
#[component(scope = "msh")]
#[component(class(bar = "msh-bar"))]
#[component(modifier(horizontal, vertical))]
#[component(base_css)]
pub struct SplitHandleStyle {
    #[prop(var = "--msh-thickness")]
    pub thickness: String,
    #[prop(var = "--msh-target-thickness")]
    pub hover_target_thickness: String,
    #[prop(var = "--msh-color")]
    pub color: String,
    #[prop(css = "background", on = bar, pseudo = ":hover")]
    pub hover_color: String,
}

impl Default for SplitHandleStyle {
    fn default() -> Self {
        SplitHandleStyle {
            thickness: "4px".into(),
            hover_target_thickness: "8px".into(),
            color: "transparent".into(),
            hover_color: "#007acc".into(),
        }
    }
}

impl css_styled::StyledComponentBase for SplitHandleStyle {
    fn base_css() -> &'static str {
        css_styled::css!(SplitHandleStyle, {
            SCOPE {
                display: flex;
                align-items: center;
                justify-content: center;
                flex-shrink: 0;
            }
            SCOPE.HORIZONTAL {
                cursor: col-resize;
                width: var(--msh-target-thickness);
            }
            SCOPE.VERTICAL {
                cursor: row-resize;
                height: var(--msh-target-thickness);
            }
            BAR {
                background: var(--msh-color);
                pointer-events: none;
            }
            SCOPE.HORIZONTAL BAR {
                width: var(--msh-thickness);
                height: 100%;
            }
            SCOPE.VERTICAL BAR {
                height: var(--msh-thickness);
                width: 100%;
            }
        })
    }
}

/// Theme for leaf panes.
#[derive(Clone, Debug)]
pub struct PaneTheme {
    /// Background color of pane content area.
    pub background: String,
    /// Text color.
    pub color: String,
}

impl Default for PaneTheme {
    fn default() -> Self {
        PaneTheme {
            background: "transparent".into(),
            color: "inherit".into(),
        }
    }
}

/// Theme for the drag-and-drop overlay.
#[derive(Clone, Debug)]
pub struct DropOverlayTheme {
    /// Background color of the drop zone indicator.
    pub indicator_color: String,
}

impl Default for DropOverlayTheme {
    fn default() -> Self {
        DropOverlayTheme {
            indicator_color: "rgba(0,122,204,0.3)".into(),
        }
    }
}

/// Top-level theme for the mullion root container.
#[derive(Clone, Debug)]
pub struct MullionTheme {
    /// Background color of the root.
    pub background: String,
}

impl Default for MullionTheme {
    fn default() -> Self {
        MullionTheme {
            background: "transparent".into(),
        }
    }
}
