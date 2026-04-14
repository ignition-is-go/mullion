use leptos::prelude::*;

use crate::activity::ActivityIcon;
use crate::context::MullionContext;
use crate::theme::MullionTheme;
use crate::tree::{ActivityId, CategoryId, PaneData, PaneId, SplitDirection};

/// Internal CSS variables for the activity bar — not exposed to consumers.
#[derive(css_styled::CssVars)]
struct ActivityBarInternal {
    #[var("--ab-cat-color")]
    pub category_color: String,
}

/// Style for the activity bar, powered by css-styled.
///
/// All customizable values are CSS custom properties. Hover behavior and
/// structural layout come from base CSS. Active/inactive opacity is applied
/// via inline styles since it varies per-button at runtime.
#[derive(css_styled::StyledComponent, Clone, Debug)]
#[component(scope = "mullion-ab")]
#[component(theme = MullionTheme)]
#[component(class(panel = "mullion-ab-panel", label = "mullion-ab-label", icon_slot = "mullion-ab-icon-slot", btn = "mullion-ab-btn", dot = "mullion-ab-dot", cat_border = "mullion-ab-cat-border", icon = "mullion-ab-icon"))]
#[component(internals(ActivityBarInternal))]
#[component(base_css)]
pub struct ActivityBarStyle {
    #[prop(var = "--ab-width", default = "28px")]
    pub width: String,
    #[prop(var = "--ab-expanded-width", default = "150px")]
    pub expanded_width: String,
    #[prop(var = "--ab-icon-size", default = "14px")]
    pub icon_size: String,
    #[prop(var = "--ab-background", default = theme.surface)]
    pub background: String,
    #[prop(var = "--ab-border", default = "1px solid var(--ml-border)")]
    pub border: String,
    #[prop(var = "--ab-border-radius", default = "0")]
    pub border_radius: String,
    #[prop(var = "--ab-expanded-padding", default = "8px")]
    pub expanded_padding: String,
    #[prop(var = "--ab-font-size", default = "11px")]
    pub font_size: String,
    #[prop(var = "--ab-icon-color", default = theme.text)]
    pub icon_color: String,
    #[prop(var = "--ab-icon-stroke-color", default = theme.text)]
    pub icon_stroke_color: String,
    #[prop(var = "--ab-icon-opacity", default = "0.5")]
    pub icon_opacity: String,
    #[prop(var = "--ab-icon-active-opacity", default = "1")]
    pub icon_active_opacity: String,
    #[prop(var = "--ab-cat-border-width", default = "2px")]
    pub category_border_width: String,
}

impl css_styled::StyledComponentBase for ActivityBarStyle {
    fn base_css() -> &'static str {
        css_styled::css!(ActivityBarStyle, {
            SCOPE {
                flex-shrink: 0;
                position: relative;
                width: var(--ab-width);
            }
            PANEL {
                position: absolute;
                top: 0;
                left: 0;
                bottom: 0;
                background: var(--ab-background);
                border-right: var(--ab-border);
                border-radius: var(--ab-border-radius);
                z-index: 10;
                display: flex;
                flex-direction: column;
                justify-content: space-between;
                overflow-y: auto;
                overflow-x: hidden;
                scrollbar-width: none;
                width: var(--ab-width);
                padding-right: 0;
                transition: width 0.15s ease, padding-right 0.15s ease;
            }
            SCOPE:hover PANEL {
                width: var(--ab-expanded-width);
                padding-right: var(--ab-expanded-padding);
            }
            LABEL {
                display: none;
                overflow: hidden;
                text-overflow: ellipsis;
            }
            SCOPE:hover LABEL {
                display: inline;
            }
            ICON_SLOT {
                width: var(--ab-width);
                flex-shrink: 0;
                display: flex;
                align-items: center;
                justify-content: center;
            }
            BTN {
                display: flex;
                align-items: center;
                height: var(--ab-width);
                cursor: pointer;
                white-space: nowrap;
                border: none;
                background: none;
                width: 100%;
                text-align: left;
                font-size: var(--ab-font-size);
                padding: 0;
                color: var(--ab-icon-color);
                opacity: var(--ab-icon-opacity);
                position: relative;
            }
            ICON {
                display: flex;
                align-items: center;
                justify-content: center;
                width: var(--ab-icon-size);
                height: var(--ab-icon-size);
                flex-shrink: 0;
                overflow: hidden;
                stroke: var(--ab-icon-stroke-color);
            }
            DOT {
                position: absolute;
                left: 2px;
                top: 50%;
                transform: translateY(-50%);
                width: 4px;
                height: 4px;
                border-radius: 50%;
                background: var(--ab-cat-color);
            }
            CAT_BORDER {
                position: absolute;
                left: 0;
                top: 0;
                bottom: 0;
                width: var(--ab-cat-border-width);
                background: var(--ab-cat-color);
            }
        })
    }
}

/// Renders the activity bar for a single pane.
///
/// Shows categories as clickable icons. On hover (pure CSS), expands to show
/// activity names. Clicking a category toggles its expanded activity list.
#[component]
pub fn ActivityBar<D: PaneData + Send + Sync>(
    pane_id: PaneId,
    data: ReadSignal<D>,
    ctx: MullionContext<D>,
    #[prop(optional)]
    app_icon: Option<ActivityIcon>,
) -> impl IntoView {
    let style = ctx.activity_bar_style.clone();
    let (expanded_cat, set_expanded_cat) = signal(Option::<CategoryId>::None);

    let ctx_for_memo = ctx.clone();
    let grouped = Memo::new(move |_| {
        let d = data.get();
        let acts = ctx_for_memo.activities_for_pane(&d);
        let cats = ctx_for_memo.sorted_categories();

        let mut groups: Vec<(CategoryId, String, ActivityIcon, String, Vec<(ActivityId, String, ActivityIcon)>)> = Vec::new();
        for cat in &cats {
            let in_cat: Vec<_> = acts
                .iter()
                .filter(|a| a.category == cat.id)
                .map(|a| (a.def.id.clone(), a.def.name.clone(), a.def.icon.clone()))
                .collect();
            if !in_cat.is_empty() {
                groups.push((cat.id.clone(), cat.name.clone(), cat.icon.clone(), cat.color.clone(), in_cat));
            }
        }
        groups
    });

    let ctx_for_active = ctx.clone();
    let pid_for_active = pane_id.clone();
    let active_activity = Memo::new(move |_| {
        let tree = ctx_for_active.tree.get();
        match tree.find(&pid_for_active) {
            Some(crate::tree::PaneNode::Leaf { active_activity, .. }) => active_activity.clone(),
            _ => None,
        }
    });

    // Auto-expand category of active activity
    let ctx_for_expand = ctx.clone();
    Effect::new(move |_| {
        let active = active_activity.get();
        if let Some(act_id) = active {
            if let Some(cat_id) = ctx_for_expand.activity_category(&act_id) {
                set_expanded_cat.set(Some(cat_id));
            }
        }
    });

    let icon_active_opacity = style.icon_active_opacity.clone();

    let ctx_actions = ctx.clone();

    view! {
        <div class=ActivityBarStyle::SCOPE>
            <div class=ActivityBarStyle::PANEL>
                // App icon + categories + activities
                <div>
                    {app_icon.map(|icon| {
                        let ctx_drag = ctx.clone();
                        let ctx_dragend = ctx.clone();
                        let pid_drag = pane_id.clone();
                        view! {
                            <div class=ActivityBarStyle::BTN
                                 style="cursor:grab"
                                 draggable="true"
                                 on:dragstart=move |ev| {
                                     ctx_drag.dragging_pane.set(Some(pid_drag.clone()));
                                     if let Some(dt) = ev.data_transfer() {
                                         let _ = dt.set_data("text/plain", &pid_drag.0);
                                         dt.set_effect_allowed("move");
                                     }
                                 }
                                 on:dragend=move |_| {
                                     ctx_dragend.dragging_pane.set(None);
                                 }>
                                <span class=ActivityBarStyle::ICON_SLOT>
                                    {render_icon(&icon)}
                                </span>
                                <span class=ActivityBarStyle::LABEL style="font-weight:600;font-size:12px"></span>
                            </div>
                        }
                    })}
                    {
                        let pane_id = pane_id.clone();
                        move || {
                        let pane_id = pane_id.clone();
                        let groups = grouped.get();
                        let current_active = active_activity.get();
                        let current_expanded = expanded_cat.get();

                        groups.into_iter().map(|(cat_id, cat_name, cat_icon, cat_color, acts)| {
                            let is_expanded = current_expanded.as_ref() == Some(&cat_id);
                            let has_active = acts.iter().any(|(id, _, _)| current_active.as_ref() == Some(id));
                            let cat_active = is_expanded || has_active;
                            let cat_style = if cat_active {
                                ActivityBarStyle::vars(|v| v.icon_opacity(&icon_active_opacity))
                            } else {
                                String::new()
                            };
                            let show_dot = !is_expanded && has_active;
                            let dot_color = cat_color.clone();
                            let cat_color_for_border = cat_color.clone();

                            let cat_id_click = cat_id.clone();
                            view! {
                                <div>
                                    <button class=ActivityBarStyle::BTN
                                            style=cat_style
                                            on:click=move |_| {
                                        if is_expanded { set_expanded_cat.set(None); }
                                        else { set_expanded_cat.set(Some(cat_id_click.clone())); }
                                    }>
                                        <span class=ActivityBarStyle::ICON_SLOT>
                                            {if show_dot {
                                                Some(view! {
                                                    <span class=ActivityBarStyle::DOT style=ActivityBarInternal::vars(|v| v.category_color(&dot_color))></span>
                                                })
                                            } else { None }}
                                            {render_icon(&cat_icon)}
                                        </span>
                                        <span class=ActivityBarStyle::LABEL>{cat_name.clone()}</span>
                                    </button>
                                    {if is_expanded {
                                        Some(view! {
                                            <div style="position:relative">
                                                <div class=ActivityBarStyle::CAT_BORDER style=ActivityBarInternal::vars(|v| v.category_color(&cat_color_for_border))></div>
                                                {acts.into_iter().map(|(act_id, name, icon)| {
                                                    let is_active = current_active.as_ref() == Some(&act_id);
                                                    let active_style = if is_active {
                                                        ActivityBarStyle::vars(|v| {
                                                            v.icon_opacity(&icon_active_opacity)
                                                             .icon_color(&cat_color_for_border)
                                                             .icon_stroke_color(&cat_color_for_border)
                                                        })
                                                    } else {
                                                        String::new()
                                                    };
                                                    let ctx = ctx.clone();
                                                    let pid = pane_id.clone();
                                                    let label = name.clone();
                                                    view! {
                                                        <button class=ActivityBarStyle::BTN
                                                                style=active_style
                                                                on:click=move |_| {
                                                            ctx.set_active_activity(&pid, Some(act_id.clone()));
                                                        }>
                                                            <span class=ActivityBarStyle::ICON_SLOT>
                                                                {render_icon(&icon)}
                                                            </span>
                                                            <span class=ActivityBarStyle::LABEL>{label.clone()}</span>
                                                        </button>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        })
                                    } else { None }}
                                </div>
                            }
                        }).collect::<Vec<_>>()
                    }}
                </div>
                // Pane actions (bottom)
                <div>
                    {
                        let ctx_sh = ctx_actions.clone();
                        let ctx_sv = ctx_actions.clone();
                        let ctx_cl = ctx_actions.clone();
                        let pid_sh = pane_id.clone();
                        let pid_sv = pane_id.clone();
                        let pid_cl = pane_id.clone();
                        view! {
                            <button class=ActivityBarStyle::BTN on:click=move |_| {
                                let d = data.get();
                                let new_id = PaneId::new(format!("{:.0}", web_sys::js_sys::Math::random() * 1e12));
                                ctx_sh.split_pane(&pid_sh, SplitDirection::Horizontal, new_id, d);
                            }>
                                <span class=ActivityBarStyle::ICON_SLOT><span class=ActivityBarStyle::ICON inner_html=ICON_SPLIT_H></span></span>
                                <span class=ActivityBarStyle::LABEL>"Split H"</span>
                            </button>
                            <button class=ActivityBarStyle::BTN on:click=move |_| {
                                let d = data.get();
                                let new_id = PaneId::new(format!("{:.0}", web_sys::js_sys::Math::random() * 1e12));
                                ctx_sv.split_pane(&pid_sv, SplitDirection::Vertical, new_id, d);
                            }>
                                <span class=ActivityBarStyle::ICON_SLOT><span class=ActivityBarStyle::ICON inner_html=ICON_SPLIT_V></span></span>
                                <span class=ActivityBarStyle::LABEL>"Split V"</span>
                            </button>
                            <button class=ActivityBarStyle::BTN on:click=move |_| { ctx_cl.close_pane(&pid_cl); }>
                                <span class=ActivityBarStyle::ICON_SLOT><span class=ActivityBarStyle::ICON inner_html=ICON_CLOSE></span></span>
                                <span class=ActivityBarStyle::LABEL>"Close"</span>
                            </button>
                        }
                    }
                </div>
            </div>
        </div>
    }
}

fn render_icon(icon: &ActivityIcon) -> AnyView {
    let icon_class = ActivityBarStyle::ICON;
    match icon {
        ActivityIcon::Class(class) => view! { <span class=format!("{} {}", icon_class, class)></span> }.into_any(),
        ActivityIcon::Svg(svg) => {
            let normalized = normalize_svg(svg);
            view! { <span class=icon_class inner_html={normalized}></span> }.into_any()
        }
        ActivityIcon::Url(url) => view! { <img class=icon_class src={url.clone()} style="object-fit:contain" /> }.into_any(),
    }
}

fn normalize_svg(svg: &str) -> String {
    let mut result = svg.to_string();
    if let Some(pos) = result.find("<svg") {
        let insert_at = pos + 4;
        result.insert_str(insert_at, " style=\"width:100%;height:100%;display:block\"");
    }
    result
}

const ICON_SPLIT_H: &str = r#"<svg viewBox="0 0 16 16" fill="currentColor"><path d="M14 1H2a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V2a1 1 0 0 0-1-1zM2 2h5.5v12H2V2zm6.5 12V2H14v12H8.5z"/></svg>"#;

const ICON_SPLIT_V: &str = r#"<svg viewBox="0 0 16 16" fill="currentColor"><path d="M14 1H2a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V2a1 1 0 0 0-1-1zM2 2h12v5.5H2V2zm0 6.5h12V14H2V8.5z"/></svg>"#;

const ICON_CLOSE: &str = r#"<svg viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.707L8.707 8l3.647-3.646-.707-.708L8 7.293 4.354 3.646l-.707.708L7.293 8l-3.646 3.646.707.708L8 8.707z"/></svg>"#;
