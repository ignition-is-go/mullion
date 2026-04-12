use leptos::prelude::*;

use crate::activity::ActivityIcon;
use crate::context::MullionContext;
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
    let theme = ctx.activity_bar_theme.clone();
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

    let bar_width = theme.width.clone();
    let expanded_width = theme.expanded_width.clone();
    let bar_bg = theme.background.clone();
    let bar_border = theme.border.clone();
    let bar_border_radius = theme.border_radius.clone();
    let expanded_padding = theme.expanded_padding.clone();
    let font_size = theme.font_size.clone();
    let icon_size = theme.icon_size.clone();
    let icon_color = theme.icon_color.clone();
    let icon_stroke_color = theme.icon_stroke_color.clone();
    let icon_opacity = theme.icon_opacity.clone();
    let icon_active_opacity = theme.icon_active_opacity.clone();
    let border_width = theme.category_border_width.clone();

    let btn_height = bar_width.clone();

    let ctx_actions = ctx.clone();

    // Clone for bottom actions (outside reactive closure)
    let action_btn_height = btn_height.clone();
    let action_icon_opacity = icon_opacity.clone();
    let action_icon_color = icon_color.clone();
    let action_font_size = font_size.clone();
    let action_icon_size = icon_size.clone();
    let action_icon_slot = format!(
        "width:{};flex-shrink:0;display:flex;align-items:center;justify-content:center",
        bar_width
    );

    // Spacer reserves collapsed width in the layout
    let spacer_style = format!("width:{};flex-shrink:0;position:relative", bar_width);

    // Inject a <style> tag with a unique scope for :hover behavior.
    // This avoids signals entirely — the browser handles hover natively.
    let scope_id = format!("mb-{}-{}", pane_id.0.replace(' ', "-"), (web_sys::js_sys::Math::random() * 1_000_000.0) as u64);
    let scope_cls = scope_id.clone();

    let css = format!(
        r#".{scope} .mb-panel {{
            width: {collapsed};
            padding-right: 0;
            transition: width 0.15s ease, padding-right 0.15s ease;
        }}
        .{scope}:hover .mb-panel {{
            width: {expanded};
            padding-right: {exp_pad};
        }}
        .{scope} .mb-label {{
            display: none;
        }}
        .{scope}:hover .mb-label {{
            display: inline;
            overflow: hidden;
            text-overflow: ellipsis;
        }}"#,
        scope = scope_cls,
        collapsed = bar_width,
        expanded = expanded_width,
        exp_pad = expanded_padding,
    );

    let icon_slot_style = format!(
        "width:{};flex-shrink:0;display:flex;align-items:center;justify-content:center",
        bar_width
    );

    let panel_style = format!(
        "position:absolute;top:0;left:0;bottom:0;background:{};border-right:{};border-radius:{};z-index:10;display:flex;flex-direction:column;justify-content:space-between;overflow-y:auto;overflow-x:hidden",
        bar_bg, bar_border, bar_border_radius
    );

    view! {
        <div class={scope_id} style={spacer_style}>
            <style>{css}</style>
            <div class="mb-panel" style={panel_style}>
                // App icon + categories + activities
                <div>
                    {app_icon.map(|icon| {
                        let slot = icon_slot_style.clone();
                        let h = btn_height.clone();
                        let app_style = format!(
                            "display:flex;align-items:center;height:{};padding:0;border:none;background:none;width:100%;cursor:grab",
                            h
                        );
                        let ctx_drag = ctx.clone();
                        let ctx_dragend = ctx.clone();
                        let pid_drag = pane_id.clone();
                        view! {
                            <div style={app_style}
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
                                <span style={slot}>
                                    {render_icon(&icon, &icon_size, &icon_color, &icon_stroke_color)}
                                </span>
                                <span class="mb-label" style="overflow:hidden;text-overflow:ellipsis;font-weight:600;font-size:12px"></span>
                            </div>
                        }
                    })}
                    {
                        let pane_id = pane_id.clone();
                        move || {
                        let pane_id = pane_id.clone(); // clone for each re-render
                        let groups = grouped.get();
                        let current_active = active_activity.get();
                        let current_expanded = expanded_cat.get();

                        groups.into_iter().map(|(cat_id, cat_name, cat_icon, cat_color, acts)| {
                            let is_expanded = current_expanded.as_ref() == Some(&cat_id);
                            let has_active = acts.iter().any(|(id, _, _)| current_active.as_ref() == Some(id));
                            let cat_opacity = if is_expanded || has_active { icon_active_opacity.clone() } else { icon_opacity.clone() };
                            let show_dot = !is_expanded && has_active;
                            let dot_color = cat_color.clone();
                            let cat_color_for_border = cat_color.clone();

                            let row_style = format!(
                                "display:flex;align-items:center;height:{};cursor:pointer;opacity:{};color:{};white-space:nowrap;border:none;background:none;width:100%;text-align:left;font-size:{};position:relative;padding:0",
                                btn_height, cat_opacity, icon_color, font_size
                            );

                            let cat_id_click = cat_id.clone();
                            view! {
                                <div>
                                    <button style={row_style.clone()} on:click=move |_| {
                                        if is_expanded { set_expanded_cat.set(None); }
                                        else { set_expanded_cat.set(Some(cat_id_click.clone())); }
                                    }>
                                        <span style={icon_slot_style.clone()}>
                                            {if show_dot {
                                                Some(view! {
                                                    <span style={format!("position:absolute;left:2px;top:50%;transform:translateY(-50%);width:4px;height:4px;border-radius:50%;background:{}", dot_color)}></span>
                                                })
                                            } else { None }}
                                            {render_icon(&cat_icon, &icon_size, &icon_color, &icon_stroke_color)}
                                        </span>
                                        <span class="mb-label" style="overflow:hidden;text-overflow:ellipsis">{cat_name.clone()}</span>
                                    </button>
                                    {if is_expanded {
                                        let line_style = format!(
                                            "position:absolute;left:0;top:0;bottom:0;width:{};background:{}",
                                            border_width, cat_color_for_border
                                        );
                                        Some(view! {
                                            <div style="position:relative">
                                                <div style={line_style}></div>
                                                {acts.into_iter().map(|(act_id, name, icon)| {
                                                    let is_active = current_active.as_ref() == Some(&act_id);
                                                    let opacity = if is_active { icon_active_opacity.clone() } else { icon_opacity.clone() };
                                                    let act_color = if is_active { cat_color_for_border.clone() } else { icon_color.clone() };
                                                    let act_stroke = if is_active { cat_color_for_border.clone() } else { icon_stroke_color.clone() };
                                                    let ctx = ctx.clone();
                                                    let pid = pane_id.clone();
                                                    let act_row_style = format!(
                                                        "display:flex;align-items:center;height:{};cursor:pointer;opacity:{};color:{};white-space:nowrap;border:none;background:none;width:100%;text-align:left;font-size:{};padding:0",
                                                        btn_height, opacity, act_color, font_size
                                                    );
                                                    let label = name.clone();
                                                    view! {
                                                        <button style={act_row_style} on:click=move |_| {
                                                            ctx.set_active_activity(&pid, Some(act_id.clone()));
                                                        }>
                                                            <span style={icon_slot_style.clone()}>
                                                                {render_icon(&icon, &icon_size, &act_color, &act_stroke)}
                                                            </span>
                                                            <span class="mb-label" style="overflow:hidden;text-overflow:ellipsis">{label.clone()}</span>
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
                        let action_style = format!(
                            "display:flex;align-items:center;height:{};cursor:pointer;opacity:{};color:{};white-space:nowrap;border:none;background:none;width:100%;font-size:{};padding:0",
                            action_btn_height, action_icon_opacity, action_icon_color, action_font_size
                        );
                        let icon_s = format!("display:flex;width:{};height:{};flex-shrink:0;overflow:hidden", action_icon_size, action_icon_size);
                        let s1 = action_style.clone();
                        let s2 = action_style.clone();
                        let s3 = action_style;
                        let is1 = icon_s.clone();
                        let is2 = icon_s.clone();
                        let is3 = icon_s;
                        let sl1 = action_icon_slot.clone();
                        let sl2 = action_icon_slot.clone();
                        let sl3 = action_icon_slot.clone();
                        let ctx_sh = ctx_actions.clone();
                        let ctx_sv = ctx_actions.clone();
                        let ctx_cl = ctx_actions.clone();
                        let pid_sh = pane_id.clone();
                        let pid_sv = pane_id.clone();
                        let pid_cl = pane_id.clone();
                        view! {
                            <button style={s1} on:click=move |_| {
                                let d = data.get();
                                let new_id = PaneId::new(format!("{:.0}", web_sys::js_sys::Math::random() * 1e12));
                                ctx_sh.split_pane(&pid_sh, SplitDirection::Horizontal, new_id, d);
                            }>
                                <span style={sl1}><span inner_html=ICON_SPLIT_H style={is1}></span></span>
                                <span class="mb-label">"Split H"</span>
                            </button>
                            <button style={s2} on:click=move |_| {
                                let d = data.get();
                                let new_id = PaneId::new(format!("{:.0}", web_sys::js_sys::Math::random() * 1e12));
                                ctx_sv.split_pane(&pid_sv, SplitDirection::Vertical, new_id, d);
                            }>
                                <span style={sl2}><span inner_html=ICON_SPLIT_V style={is2}></span></span>
                                <span class="mb-label">"Split V"</span>
                            </button>
                            <button style={s3} on:click=move |_| { ctx_cl.close_pane(&pid_cl); }>
                                <span style={sl3}><span inner_html=ICON_CLOSE style={is3}></span></span>
                                <span class="mb-label">"Close"</span>
                            </button>
                        }
                    }
                </div>
            </div>
        </div>
    }
}

fn render_icon(icon: &ActivityIcon, size: &str, fill_color: &str, stroke_color: &str) -> AnyView {
    let style = format!(
        "display:flex;align-items:center;justify-content:center;width:{};height:{};flex-shrink:0;overflow:hidden;color:{}",
        size, size, fill_color
    );
    match icon {
        ActivityIcon::Class(class) => view! { <span class={class.clone()} style={style}></span> }.into_any(),
        ActivityIcon::Svg(svg) => {
            let normalized = normalize_svg(svg, stroke_color);
            view! { <span inner_html={normalized} style={style}></span> }.into_any()
        }
        ActivityIcon::Url(url) => view! { <img src={url.clone()} style={format!("{};object-fit:contain", style)} /> }.into_any(),
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
