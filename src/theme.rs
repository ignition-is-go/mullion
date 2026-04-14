/// Style for the activity bar, powered by css-styled.
///
/// All customizable values are CSS custom properties. Hover behavior and
/// structural layout come from base CSS. Active/inactive opacity is applied
/// via inline styles since it varies per-button at runtime.
#[derive(css_styled::StyledComponent, Clone, Debug)]
#[component(scope = "mullion-ab")]
#[component(class(panel = "mullion-ab-panel", label = "mullion-ab-label", icon_slot = "mullion-ab-icon-slot", btn = "mullion-ab-btn", dot = "mullion-ab-dot", cat_border = "mullion-ab-cat-border", icon = "mullion-ab-icon"))]
#[component(base_css)]
pub struct ActivityBarStyle {
    #[prop(var = "--ab-width")]
    pub width: String,
    #[prop(var = "--ab-expanded-width")]
    pub expanded_width: String,
    #[prop(var = "--ab-icon-size")]
    pub icon_size: String,
    #[prop(var = "--ab-background")]
    pub background: String,
    #[prop(var = "--ab-border")]
    pub border: String,
    #[prop(var = "--ab-border-radius")]
    pub border_radius: String,
    #[prop(var = "--ab-expanded-padding")]
    pub expanded_padding: String,
    #[prop(var = "--ab-font-size")]
    pub font_size: String,
    #[prop(var = "--ab-icon-color")]
    pub icon_color: String,
    #[prop(var = "--ab-icon-stroke-color")]
    pub icon_stroke_color: String,
    #[prop(var = "--ab-icon-opacity")]
    pub icon_opacity: String,
    #[prop(var = "--ab-icon-active-opacity")]
    pub icon_active_opacity: String,
    #[prop(var = "--ab-cat-border-width")]
    pub category_border_width: String,
}

impl Default for ActivityBarStyle {
    fn default() -> Self {
        ActivityBarStyle {
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

impl css_styled::StyledComponentBase for ActivityBarStyle {
    fn base_css() -> &'static str {
        static CSS: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        CSS.get_or_init(|| format!(
            "\
.{scope} {{ flex-shrink: 0; position: relative; width: var(--ab-width); }}\n\
.{panel} {{ position: absolute; top: 0; left: 0; bottom: 0; background: var(--ab-background); border-right: var(--ab-border); border-radius: var(--ab-border-radius); z-index: 10; display: flex; flex-direction: column; justify-content: space-between; overflow-y: auto; overflow-x: hidden; scrollbar-width: none; width: var(--ab-width); padding-right: 0; transition: width 0.15s ease, padding-right 0.15s ease; }}\n\
.{panel}::-webkit-scrollbar {{ display: none; }}\n\
.{scope}:hover .{panel} {{ width: var(--ab-expanded-width); padding-right: var(--ab-expanded-padding); }}\n\
.{label} {{ display: none; overflow: hidden; text-overflow: ellipsis; }}\n\
.{scope}:hover .{label} {{ display: inline; }}\n\
.{icon_slot} {{ width: var(--ab-width); flex-shrink: 0; display: flex; align-items: center; justify-content: center; }}\n\
.{btn} {{ display: flex; align-items: center; height: var(--ab-width); cursor: pointer; white-space: nowrap; border: none; background: none; width: 100%; text-align: left; font-size: var(--ab-font-size); padding: 0; color: var(--ab-icon-color); opacity: var(--ab-icon-opacity); }}\n\
.{icon} {{ display: flex; align-items: center; justify-content: center; width: var(--ab-icon-size); height: var(--ab-icon-size); flex-shrink: 0; overflow: hidden; }}\n\
.{dot} {{ position: absolute; left: 2px; top: 50%; transform: translateY(-50%); width: 4px; height: 4px; border-radius: 50%; }}\n\
.{cat_border} {{ position: absolute; left: 0; top: 0; bottom: 0; width: var(--ab-cat-border-width); }}",
            scope = Self::SCOPE,
            panel = Self::PANEL,
            label = Self::LABEL,
            icon_slot = Self::ICON_SLOT,
            btn = Self::BTN,
            icon = Self::ICON,
            dot = Self::DOT,
            cat_border = Self::CAT_BORDER,
        )).as_str()
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

/// Style for leaf panes, powered by css-styled.
#[derive(css_styled::StyledComponent, Clone, Debug)]
#[component(scope = "mullion-pane")]
#[component(base_css)]
pub struct PaneStyle {
    #[prop(css = "background")]
    pub background: String,
    #[prop(css = "color")]
    pub color: String,
}

impl Default for PaneStyle {
    fn default() -> Self {
        PaneStyle {
            background: "transparent".into(),
            color: "inherit".into(),
        }
    }
}

impl css_styled::StyledComponentBase for PaneStyle {
    fn base_css() -> &'static str {
        css_styled::css!(PaneStyle, {
            SCOPE {
                display: flex;
                flex-direction: row;
                width: 100%;
                height: 100%;
                overflow: hidden;
            }
        })
    }
}

/// Style for the drag-and-drop overlay, powered by css-styled.
///
/// The indicator color is a CSS variable applied to the base styling.
/// Edge-specific positioning (left/right/top/bottom/width/height) stays
/// as inline styles since it is driven by runtime drag state.
#[derive(css_styled::StyledComponent, Clone, Debug)]
#[component(scope = "mullion-drop")]
#[component(base_css)]
pub struct DropOverlayStyle {
    #[prop(var = "--drop-indicator-color")]
    pub indicator_color: String,
}

impl Default for DropOverlayStyle {
    fn default() -> Self {
        DropOverlayStyle {
            indicator_color: "rgba(0,122,204,0.3)".into(),
        }
    }
}

impl css_styled::StyledComponentBase for DropOverlayStyle {
    fn base_css() -> &'static str {
        css_styled::css!(DropOverlayStyle, {
            SCOPE {
                position: absolute;
                pointer-events: none;
                background: var(--drop-indicator-color);
            }
        })
    }
}

/// Style for the mullion root container, powered by css-styled.
#[derive(css_styled::StyledComponent, Clone, Debug)]
#[component(scope = "mullion-root")]
#[component(base_css)]
pub struct MullionStyle {
    #[prop(css = "background")]
    pub background: String,
}

impl Default for MullionStyle {
    fn default() -> Self {
        MullionStyle {
            background: "transparent".into(),
        }
    }
}

impl css_styled::StyledComponentBase for MullionStyle {
    fn base_css() -> &'static str {
        css_styled::css!(MullionStyle, {
            SCOPE {
                width: 100%;
                height: 100%;
            }
        })
    }
}
