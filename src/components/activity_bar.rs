use leptos::prelude::*;

use crate::activity::ActivityIcon;
use crate::context::MullionContext;
use crate::theme::ActivityBarStyle;
use crate::tree::{ActivityId, CategoryId, PaneData, PaneId, SplitDirection};

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

    let icon_stroke_color = style.icon_stroke_color.clone();
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
                                    {render_icon(&icon, &icon_stroke_color)}
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
                                format!("opacity:{};position:relative", icon_active_opacity)
                            } else {
                                "position:relative".to_string()
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
                                                    <span class=ActivityBarStyle::DOT style=format!("background:{}", dot_color)></span>
                                                })
                                            } else { None }}
                                            {render_icon(&cat_icon, &icon_stroke_color)}
                                        </span>
                                        <span class=ActivityBarStyle::LABEL>{cat_name.clone()}</span>
                                    </button>
                                    {if is_expanded {
                                        Some(view! {
                                            <div style="position:relative">
                                                <div class=ActivityBarStyle::CAT_BORDER style=format!("background:{}", cat_color_for_border)></div>
                                                {acts.into_iter().map(|(act_id, name, icon)| {
                                                    let is_active = current_active.as_ref() == Some(&act_id);
                                                    let active_style = if is_active {
                                                        format!("opacity:{};color:{}", icon_active_opacity, cat_color_for_border)
                                                    } else {
                                                        String::new()
                                                    };
                                                    let act_stroke = if is_active { cat_color_for_border.clone() } else { icon_stroke_color.clone() };
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
                                                                {render_icon(&icon, &act_stroke)}
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

fn render_icon(icon: &ActivityIcon, stroke_color: &str) -> AnyView {
    let icon_class = ActivityBarStyle::ICON;
    match icon {
        ActivityIcon::Class(class) => view! { <span class=format!("{} {}", icon_class, class)></span> }.into_any(),
        ActivityIcon::Svg(svg) => {
            let normalized = normalize_svg(svg, stroke_color);
            view! { <span class=icon_class inner_html={normalized}></span> }.into_any()
        }
        ActivityIcon::Url(url) => view! { <img class=icon_class src={url.clone()} style="object-fit:contain" /> }.into_any(),
    }
}

fn normalize_svg(svg: &str, stroke_color: &str) -> String {
    let mut result = svg.to_string();
    if let Some(pos) = result.find("<svg") {
        let insert_at = pos + 4;
        let style_attr = format!(" style=\"width:100%;height:100%;display:block;stroke:{}\"", stroke_color);
        result.insert_str(insert_at, &style_attr);
    }
    result
}

const ICON_SPLIT_H: &str = r#"<svg viewBox="0 0 16 16" fill="currentColor"><path d="M14 1H2a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V2a1 1 0 0 0-1-1zM2 2h5.5v12H2V2zm6.5 12V2H14v12H8.5z"/></svg>"#;

const ICON_SPLIT_V: &str = r#"<svg viewBox="0 0 16 16" fill="currentColor"><path d="M14 1H2a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V2a1 1 0 0 0-1-1zM2 2h12v5.5H2V2zm0 6.5h12V14H2V8.5z"/></svg>"#;

const ICON_CLOSE: &str = r#"<svg viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.707L8.707 8l3.647-3.646-.707-.708L8 7.293 4.354 3.646l-.707.708L7.293 8l-3.646 3.646.707.708L8 8.707z"/></svg>"#;
