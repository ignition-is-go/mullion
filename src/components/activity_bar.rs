use leptos::prelude::*;

use crate::activity::ActivityIcon;
use crate::context::MullionContext;
use crate::drag::DragPayload;
use crate::theme::MullionTheme;
use crate::tree::{ActivityId, CategoryId, PaneData, PaneId, SplitDirection};


/// Internal CSS variables for the activity bar — not exposed to consumers.
#[derive(css_styled::CssVars)]
struct ActivityBarInternal {
    #[var("--ab-cat-color")]
    pub category_color: String,
}

/// Behavior options for the activity bar. Provide via Leptos context before
/// mounting `MullionProvider` or `MullionRoot` to override defaults.
///
/// Unlike `ActivityBarStyle` (which controls appearance via CSS variables),
/// this struct controls interaction semantics that can't be expressed as a
/// single CSS property.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivityBarBehavior {
    /// When `true` (the default), the activity bar widens on hover to reveal
    /// activity labels. Set to `false` to pin it at its collapsed width —
    /// useful when labels would overflow surrounding UI, or when the host
    /// app wants a purely icon-driven bar.
    ///
    /// Auto-hiding a bar off the pane edge is a *per-pane* decision, not a
    /// global one — see [`crate::context::PaneAutoHideActivityBar`].
    pub hover_expand: bool,
}

impl Default for ActivityBarBehavior {
    fn default() -> Self {
        Self { hover_expand: true }
    }
}

/// Style for the activity bar, powered by css-styled.
///
/// All customizable values are CSS custom properties. Hover behavior and
/// structural layout come from base CSS. Active/inactive opacity is applied
/// via inline styles since it varies per-button at runtime.
#[derive(css_styled::StyledComponent, Clone, Debug)]
#[component(scope = "mullion-ab")]
#[component(theme = MullionTheme)]
#[component(class(
    panel = "mullion-ab-panel",
    label = "mullion-ab-label",
    icon_slot = "mullion-ab-icon-slot",
    btn = "mullion-ab-btn",
    dot = "mullion-ab-dot",
    cat_border = "mullion-ab-cat-border",
    icon = "mullion-ab-icon"
))]
#[component(modifier(collapsed, auto_hide, dragging))]
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
                transition: width 0.15s ease, padding-right 0.15s ease, transform 0.15s ease;
            }
            /* Hold the bar open for the duration of a drag. Chrome drops
               `:hover` as soon as a native drag starts, so without this the
               panel collapses mid-gesture and the row being dragged shrinks out
               from under the cursor. That is not what was cancelling drags (see
               the `request_animation_frame` note on the drag source for the real
               cause) — but a drag source that resizes underneath the pointer is a
               hazard worth removing, and holding the bar open also keeps the drag
               image stable. */
            SCOPE.DRAGGING PANEL {
                width: var(--ab-expanded-width);
                transition: none;
            }
            SCOPE.DRAGGING LABEL {
                opacity: 1;
                transition: none;
            }
            SCOPE:not(.COLLAPSED):hover PANEL {
                width: var(--ab-expanded-width);
                padding-right: var(--ab-expanded-padding);
            }
            /* Auto-hide: the bar reserves no space and its panel sits fully off the
               left edge (clipped by the pane's overflow:hidden, so it never bleeds
               into a neighbouring pane). An invisible edge strip (::before) is the
               hover target that summons it — hovering the strip slides the panel in
               over the pane content; it stays open while the cursor is over the
               revealed panel, and slides back out on leave. */
            SCOPE.AUTO_HIDE {
                width: 0;
            }
            SCOPE.AUTO_HIDE::before {
                content: "";
                position: absolute;
                left: 0;
                top: 0;
                bottom: 0;
                width: 12px;
                z-index: 9;
            }
            SCOPE.AUTO_HIDE PANEL {
                transform: translateX(-100%);
            }
            SCOPE.AUTO_HIDE:hover PANEL {
                transform: translateX(0);
            }
            LABEL {
                /* Hidden by zero width + opacity rather than `display:none`, so
                   the label never leaves the box tree while a drag is in flight.
                   `flex-shrink` takes it to 0px against the fixed-width icon slot
                   when the bar is collapsed and `overflow:hidden` clips the text,
                   so it is invisible either way. Not a drag source — the row is,
                   because the row is the element that is always rendered. */
                opacity: 0;
                min-width: 0;
                overflow: hidden;
                text-overflow: ellipsis;
                transition: opacity 0.15s ease;
            }
            SCOPE:not(.COLLAPSED):hover LABEL {
                opacity: 1;
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
                /* Without this, mousedown on a label's text starts a text
                   selection, which pre-empts the HTML5 drag — so an activity
                   would only be draggable by its icon, never by its name.
                   Toolbar chrome shouldn't be selectable anyway. */
                user-select: none;
                /* `draggable="true"` alone is not enough for a text-bearing
                   element in Chrome: the text still wins the mousedown, so an
                   activity could be dragged by its icon but never by its name.
                   `-webkit-user-drag: element` makes the whole element the drag
                   source explicitly. Non-standard but supported in Chrome and
                   Safari; harmless elsewhere. */
                -webkit-user-drag: element;
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
    data: Signal<D>,
    ctx: MullionContext<D>,
    #[prop(optional)] app_icon: Option<ActivityIcon>,
    /// When `true`, this pane's bar tucks off the left edge and reveals on
    /// edge-hover (resolved per-pane by the host). Default: pinned/visible.
    #[prop(default = false)]
    auto_hide: bool,
) -> impl IntoView {
    let style = ctx.activity_bar_style.clone();
    let (expanded_cat, set_expanded_cat) = signal(Option::<CategoryId>::None);

    let ctx_for_memo = ctx.clone();
    let grouped = Memo::new(move |_| {
        let d = data.get();
        let acts = ctx_for_memo.activities_for_pane(&d);
        let cats = ctx_for_memo.sorted_categories();

        let mut groups: Vec<(
            CategoryId,
            String,
            ActivityIcon,
            String,
            Vec<(ActivityId, String, ActivityIcon)>,
        )> = Vec::new();
        for cat in &cats {
            let in_cat: Vec<_> = acts
                .iter()
                .filter(|a| a.category.as_ref() == Some(&cat.id))
                .map(|a| (a.def.id.clone(), a.def.name.clone(), a.def.icon.clone()))
                .collect();
            if !in_cat.is_empty() {
                groups.push((
                    cat.id.clone(),
                    cat.name.clone(),
                    cat.icon.clone(),
                    cat.color.clone(),
                    in_cat,
                ));
            }
        }
        groups
    });

    // Free-floating activities (registered outside any category) — rendered as
    // top-level icons that select directly, with no category expansion.
    let ctx_for_float = ctx.clone();
    let floating = Memo::new(move |_| {
        let d = data.get();
        ctx_for_float
            .activities_for_pane(&d)
            .into_iter()
            .filter(|a| a.category.is_none())
            .map(|a| (a.def.id.clone(), a.def.name.clone(), a.def.icon.clone()))
            .collect::<Vec<(ActivityId, String, ActivityIcon)>>()
    });

    let ctx_for_active = ctx.clone();
    let pid_for_active = pane_id.clone();
    let active_activity = Memo::new(move |_| {
        ctx_for_active
            .tree
            .with(|tree| match tree.find(&pid_for_active) {
                Some(crate::tree::PaneNode::Leaf {
                    active_activity, ..
                }) => active_activity.clone(),
                _ => None,
            })
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
    let icon_active_opacity_float = icon_active_opacity.clone();
    // Floating activities have no category to colour their active highlight, so the
    // selected one highlights in the theme accent (matching the categorised behaviour).
    let float_accent = ctx.theme.accent.clone();

    let ctx_actions = ctx.clone();
    let ctx_float = ctx.clone();
    let pid_float = pane_id.clone();

    // Host-provided per-pane chrome (e.g. session indicator). Cloned out before
    // `ctx` is moved into the activity-groups closure below.
    let pane_accessory = ctx.pane_accessory.clone();
    let pid_accessory = pane_id.clone();

    // Reactive on `drag`: while a drag is in flight the bar gets the `dragging`
    // modifier so it stays open (see the CSS for why that is load-bearing, not
    // cosmetic). This is a class swap on an existing element, not a re-render —
    // replacing the node mid-drag would cancel the drag just as surely.
    let hover_expand = ctx.activity_bar_behavior.hover_expand;
    let ctx_drag_class = ctx.clone();
    let scope_class = move || {
        let mut mods = Vec::new();
        if !hover_expand {
            mods.push(ActivityBarModifier::Collapsed);
        }
        if auto_hide {
            mods.push(ActivityBarModifier::AutoHide);
        }
        if ctx_drag_class.drag.get().is_some() {
            mods.push(ActivityBarModifier::Dragging);
        }
        if mods.is_empty() {
            ActivityBarStyle::SCOPE.to_string()
        } else {
            ActivityBarStyle::class(&mods)
        }
    };

    view! {
        <div class=scope_class>
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
                                     ctx_drag.drag.set(Some(DragPayload::Pane(pid_drag.clone())));
                                     if let Some(dt) = ev.data_transfer() {
                                         let _ = dt.set_data("text/plain", &pid_drag.0);
                                         dt.set_effect_allowed("move");
                                     }
                                 }
                                 on:dragend=move |_| {
                                     ctx_dragend.drag.set(None);
                                 }>
                                <span class=ActivityBarStyle::ICON_SLOT>
                                    {render_icon(&icon)}
                                </span>
                                <span class=ActivityBarStyle::LABEL style="font-weight:600;font-size:12px"></span>
                            </div>
                        }
                    })}
                    // Free-floating activities: top-level icons, select directly.
                    {
                        move || {
                            let current_active = active_activity.get();
                            floating.get().into_iter().map(|(act_id, name, icon)| {
                                let is_active = current_active.as_ref() == Some(&act_id);
                                let active_style = if is_active {
                                    ActivityBarStyle::vars(|v| {
                                        v.icon_opacity(&icon_active_opacity_float)
                                            .icon_color(&float_accent)
                                            .icon_stroke_color(&float_accent)
                                    })
                                } else {
                                    String::new()
                                };
                                let ctx = ctx_float.clone();
                                let pid = pid_float.clone();
                                let label = name.clone();
                                // A `div role=button`, not a `<button>`: form
                                // controls consume mousedown for activation, so
                                // browsers won't reliably start an HTML5 drag
                                // from one (Firefox ignores `draggable` on them
                                // outright). The app icon above is a div for the
                                // same reason. Keyboard activation is restored
                                // with tabindex + Enter/Space below.
                                let ctx_ds = ctx_float.clone();
                                let ctx_de = ctx_float.clone();
                                let ctx_key = ctx_float.clone();
                                let act_drag = act_id.clone();
                                let act_key = act_id.clone();
                                let pid_key = pid_float.clone();
                                let can_drag = ctx_float.new_pane.is_some();
                                view! {
                                    <div class=ActivityBarStyle::BTN
                                            role="button"
                                            tabindex="0"
                                            style=active_style
                                            draggable=can_drag.then_some("true")
                                            on:dragstart=move |ev| {
                                                if !can_drag { return }
                                                if let Some(dt) = ev.data_transfer() {
                                                    let _ = dt.set_data("text/plain", &act_drag.0);
                                                    dt.set_effect_allowed("copy");
                                                }
                                                // Deferred out of the dragstart handler on purpose.
                                                // Setting this mounts every pane's DropOverlay, and
                                                // an element appearing under the pointer while the
                                                // drag session is still being established makes
                                                // Chrome abandon the drag: dragstart fires, then
                                                // dragend with dropEffect=none and no dragover
                                                // anywhere, not even a document-level capture
                                                // listener. Dragging by the icon escaped it only
                                                // because the cursor sits left of the content area
                                                // where nothing is inserted.
                                                let ctx_deferred = ctx_ds.clone();
                                                let payload = DragPayload::NewActivity(act_drag.clone());
                                                request_animation_frame(move || {
                                                    ctx_deferred.drag.set(Some(payload));
                                                });
                                            }
                                            on:dragend=move |ev: web_sys::DragEvent| {
                                                let eff = ev.data_transfer()
                                                    .map(|dt| dt.drop_effect())
                                                    .unwrap_or_default();
                                                web_sys::console::log_1(
                                                    &format!("[ml-dbg] dragend     dropEffect={eff}").into(),
                                                );
                                                ctx_de.drag.set(None);
                                            }
                                            on:keydown=move |ev: web_sys::KeyboardEvent| {
                                                if ev.key() == "Enter" || ev.key() == " " {
                                                    ev.prevent_default();
                                                    ctx_key.set_active_activity(&pid_key, Some(act_key.clone()));
                                                }
                                            }
                                            on:click=move |_| {
                                        ctx.set_active_activity(&pid, Some(act_id.clone()));
                                    }>
                                        // Neither span is a drag source: the row
                                        // above owns the drag. Chrome resolves
                                        // the source by walking up from the
                                        // mousedown target, and the row is the
                                        // only ancestor that is always rendered
                                        // (the label is hover-gated).
                                        <span class=ActivityBarStyle::ICON_SLOT>
                                            {render_icon(&icon)}
                                        </span>
                                        <span class=ActivityBarStyle::LABEL>
                                            {label.clone()}
                                        </span>
                                    </div>
                                }
                            }).collect::<Vec<_>>()
                        }
                    }
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
                                                    // `div role=button` rather than
                                                    // `<button>` so an HTML5 drag can
                                                    // actually start — see the floating
                                                    // branch above for why.
                                                    let ctx_ds = ctx.clone();
                                                    let ctx_de = ctx.clone();
                                                    let ctx_key = ctx.clone();
                                                    let act_drag = act_id.clone();
                                                    let act_key = act_id.clone();
                                                    let pid_key = pane_id.clone();
                                                    let can_drag = ctx.new_pane.is_some();
                                                    let ctx = ctx.clone();
                                                    let pid = pane_id.clone();
                                                    let label = name.clone();
                                                    view! {
                                                        <div class=ActivityBarStyle::BTN
                                                                role="button"
                                                                tabindex="0"
                                                                style=active_style
                                                                draggable=can_drag.then_some("true")
                                                                on:dragstart=move |ev| {
                                                                    if !can_drag { return }
                                                                    if let Some(dt) = ev.data_transfer() {
                                                                        let _ = dt.set_data("text/plain", &act_drag.0);
                                                                        dt.set_effect_allowed("copy");
                                                                    }
                                                                    // Deferred — see the floating branch for why
                                                                    // mounting the overlays inside dragstart kills
                                                                    // the drag.
                                                                    let ctx_deferred = ctx_ds.clone();
                                                                    let payload = DragPayload::NewActivity(act_drag.clone());
                                                                    request_animation_frame(move || {
                                                                        ctx_deferred.drag.set(Some(payload));
                                                                    });
                                                                }
                                                                on:dragend=move |ev: web_sys::DragEvent| {
                                                                    let eff = ev.data_transfer()
                                                                        .map(|dt| dt.drop_effect())
                                                                        .unwrap_or_default();
                                                                    web_sys::console::log_1(
                                                                        &format!("[ml-dbg] dragend     dropEffect={eff}").into(),
                                                                    );
                                                                    ctx_de.drag.set(None);
                                                                }
                                                                on:keydown=move |ev: web_sys::KeyboardEvent| {
                                                                    if ev.key() == "Enter" || ev.key() == " " {
                                                                        ev.prevent_default();
                                                                        ctx_key.set_active_activity(&pid_key, Some(act_key.clone()));
                                                                    }
                                                                }
                                                                on:click=move |_| {
                                                            ctx.set_active_activity(&pid, Some(act_id.clone()));
                                                        }>
                                                            // Not drag sources — the
                                                            // row owns the drag; see
                                                            // the floating branch.
                                                            <span class=ActivityBarStyle::ICON_SLOT>
                                                                {render_icon(&icon)}
                                                            </span>
                                                            <span class=ActivityBarStyle::LABEL>
                                                                {label.clone()}
                                                            </span>
                                                        </div>
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
                    // Host-provided per-pane chrome (session indicator, etc.),
                    // anchored above the split/close controls.
                    {pane_accessory.map(move |f| f(pid_accessory.clone()))}
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

#[cfg(test)]
mod tests {
    use super::*;
    use css_styled::IntoCss;

    #[test]
    fn collapsed_modifier_gates_hover_rules() {
        let css = ActivityBarStyle::default().to_css();
        // The :not(.collapsed) guard must appear on both hover rules so the
        // bar stays at its collapsed width when the modifier is applied.
        assert!(
            css.contains(":not(.collapsed):hover"),
            "expected :not(.collapsed):hover in base CSS, got: {css}"
        );
        // And a plain SCOPE:hover rule must not slip through — otherwise the
        // modifier wouldn't actually suppress the hover behavior.
        assert!(
            !css.contains(".mullion-ab:hover"),
            "unguarded .mullion-ab:hover rule present in base CSS: {css}"
        );
    }

    #[test]
    fn dragging_modifier_holds_the_bar_open() {
        let css = ActivityBarStyle::default().to_css();
        // The bar must stay expanded while a drag is in flight — Chrome drops
        // `:hover` when a native drag starts, so the hover rule alone would let
        // the panel collapse and resize the drag source under the cursor.
        assert!(
            css.contains(".dragging"),
            "expected a .dragging rule in base CSS, got: {css}"
        );
        // And it must not animate: a transitioning collapse is the same hazard,
        // just spread over 150ms.
        let dragging_block = css
            .split(".dragging")
            .nth(1)
            .expect("`.dragging` rule present");
        assert!(
            dragging_block.contains("transition:none") || dragging_block.contains("transition: none"),
            "expected transition:none in the .dragging rule, got: {dragging_block}"
        );
    }

    #[test]
    fn dragging_class_has_expected_name() {
        assert_eq!(
            ActivityBarStyle::class(&[ActivityBarModifier::Dragging]),
            "mullion-ab dragging",
        );
    }

    #[test]
    fn collapsed_class_has_expected_name() {
        assert_eq!(
            ActivityBarStyle::class(&[ActivityBarModifier::Collapsed]),
            "mullion-ab collapsed",
        );
    }

    #[test]
    fn behavior_defaults_to_hover_expand_true() {
        assert!(ActivityBarBehavior::default().hover_expand);
    }
}

fn render_icon(icon: &ActivityIcon) -> AnyView {
    let icon_class = ActivityBarStyle::ICON;
    match icon {
        ActivityIcon::Class(class) => {
            view! { <span class=format!("{} {}", icon_class, class)></span> }.into_any()
        }
        ActivityIcon::Svg(svg) => {
            let normalized = normalize_svg(svg);
            view! { <span class=icon_class inner_html={normalized}></span> }.into_any()
        }
        ActivityIcon::Url(url) => {
            view! { <img class=icon_class src={url.clone()} style="object-fit:contain" /> }
                .into_any()
        }
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
