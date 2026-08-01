use leptos::prelude::*;
use leptos_resize_handle::{use_drag, Direction as LrhDirection};

use crate::context::MullionContext;
use crate::drag::DragPayload;
use crate::focus::PaneFocusBehavior;
use crate::theme::MullionTheme;
use crate::tree::{
    collect_split_keys, find_split_direction, leaf_rect, split_parent_rect, ActivityId, PaneData,
    PaneId, PaneNode, Rect, SplitDirection,
};

/// Style for leaf panes, powered by css-styled.
#[derive(css_styled::StyledComponent, Clone, Debug)]
#[component(scope = "mullion-pane")]
#[component(theme = MullionTheme)]
#[component(base_css)]
pub struct PaneStyle {
    #[prop(css = "background", default = theme.surface)]
    pub background: String,
    #[prop(css = "color", default = theme.text)]
    pub color: String,
}

impl css_styled::StyledComponentBase for PaneStyle {
    fn base_css() -> &'static str {
        css_styled::css!(PaneStyle, {
            SCOPE {
                display: flex;
                flex-direction: row;
                position: relative;
                width: 100%;
                height: 100%;
                overflow: hidden;
            }
        })
    }
}

use super::activity_bar::{ActivityBar, ActivityBarEdge};
use super::drop_overlay::DropOverlay;
use super::pane_content::PaneContent;
use super::split_handle::{SplitHandleModifier, SplitHandleStyle};

/// Renders the pane tree for a `MullionContext`.
///
/// Layout model: **flat** rather than nested. All leaves are rendered as
/// absolutely-positioned siblings inside a single `position: relative`
/// container, each sized from a `Memo<Rect>` that walks the tree. Split
/// handles are rendered the same way at split boundaries. Because leaves
/// are rendered via `<For keyed=pane_id>`, their component instances
/// (and the DOM underneath — including live WebRTC elements) are preserved
/// across structural mutations. Only newly-added leaves mount; only newly-
/// removed leaves unmount; everyone else keeps their state.
///
/// Fine-grained reactivity:
/// - Each leaf's `Rect` memo reads ratios only along its own ancestor chain,
///   so resizing an unrelated split doesn't invalidate it.
/// - Each leaf's `data` and `active_activity` memos read only the matching
///   leaf's fields, so mutations to other leaves don't re-render this one.
#[component]
pub fn PaneView<D: PaneData + Send + Sync>(ctx: MullionContext<D>) -> impl IntoView {
    let ctx_leaves = ctx.clone();
    let leaves = Memo::new(move |_| ctx_leaves.tree.with(|t| t.leaf_ids()));

    let ctx_splits = ctx.clone();
    let splits = Memo::new(move |_| {
        if ctx_splits.zoomed_pane.get().is_some() {
            Vec::new()
        } else {
            ctx_splits.tree.with(|t| collect_split_keys(t))
        }
    });

    let container_ref: NodeRef<leptos::html::Div> = NodeRef::new();

    let ctx_for_leaves = ctx.clone();
    let ctx_for_splits = ctx.clone();

    view! {
        <div
            node_ref=container_ref
            style="position:relative;width:100%;height:100%;overflow:hidden"
        >
            <For
                each=move || leaves.get()
                key=|id| id.clone()
                children=move |id| {
                    let ctx = ctx_for_leaves.clone();
                    view! { <LeafSlot id=id ctx=ctx /> }
                }
            />
            <For
                each=move || splits.get()
                key=|k| k.clone()
                children=move |split_key| {
                    let ctx = ctx_for_splits.clone();
                    view! { <SplitHandleSlot split_key=split_key ctx=ctx container_ref=container_ref /> }
                }
            />
        </div>
    }
}

#[component]
fn LeafSlot<D: PaneData + Send + Sync>(id: PaneId, ctx: MullionContext<D>) -> impl IntoView {
    let id_rect = id.clone();
    let ctx_rect = ctx.clone();
    let rect = Memo::new(move |prev: Option<&Rect>| {
        ctx_rect.tree.with(|tree| {
            let ctx_for_ratio = ctx_rect.clone();
            leaf_rect(tree, &id_rect, move |key| {
                ctx_for_ratio.ratio_signal(key).get()
            })
            .unwrap_or_else(|| prev.copied().unwrap_or(Rect::FULL))
        })
    });

    let zoomed_pane = ctx.zoomed_pane;
    let id_for_zoom = id.clone();
    let slot_style = move || {
        if let Some(zoomed) = zoomed_pane.get() {
            if zoomed == id_for_zoom {
                return "position:absolute;inset:0;display:flex;overflow:hidden;z-index:1"
                    .to_string();
            }
            return "position:absolute;inset:0;display:flex;overflow:hidden;visibility:hidden;pointer-events:none"
                .to_string();
        }
        let r = rect.get();
        format!(
            "position:absolute;left:{}%;top:{}%;width:{}%;height:{}%;display:flex;overflow:hidden",
            r.left * 100.0,
            r.top * 100.0,
            r.width * 100.0,
            r.height * 100.0,
        )
    };

    view! {
        <div style=slot_style>
            <LeafView id=id ctx=ctx rect=rect />
        </div>
    }
}

#[component]
fn LeafView<D: PaneData + Send + Sync>(
    id: PaneId,
    ctx: MullionContext<D>,
    rect: Memo<Rect>,
) -> impl IntoView {
    // Per-leaf reactive slices of the tree. Each Memo fires only when the
    // specific leaf's field changes (PartialEq dedup).
    //
    // These memos may fire AFTER the leaf has been removed from the tree
    // (during close_pane's subscriber-notification phase, before the
    // top-level `leaves` memo has re-rendered and disposed the old
    // subscribers via `<For>`). They must NOT panic — fall back to the
    // previous cached value; the slot will be unmounted moments later.
    let id_data = id.clone();
    let ctx_data = ctx.clone();
    let data_memo = Memo::new(move |prev: Option<&D>| {
        ctx_data.tree.with(|t| match t.find(&id_data) {
            Some(PaneNode::Leaf { data, .. }) => data.clone(),
            _ => prev
                .cloned()
                .expect("leaf must exist on first render of its leaf view"),
        })
    });
    let data: Signal<D> = data_memo.into();

    let id_act = id.clone();
    let ctx_act = ctx.clone();
    let activity_memo = Memo::new(move |prev: Option<&Option<ActivityId>>| {
        ctx_act.tree.with(|t| match t.find(&id_act) {
            Some(PaneNode::Leaf {
                active_activity, ..
            }) => active_activity.clone(),
            _ => prev.cloned().unwrap_or(None),
        })
    });
    let active_activity: Signal<Option<ActivityId>> = activity_memo.into();

    let ctx_focus = ctx.clone();
    let ctx_click_focus = ctx.clone();
    let ctx_ref = ctx.clone();
    let pane_ref: NodeRef<leptos::html::Div> = NodeRef::new();

    let id_for_ref = id.clone();
    pane_ref.on_load(move |el| {
        let html_el: web_sys::HtmlElement = el.into();
        ctx_ref.register_pane_element(id_for_ref.clone(), html_el);
    });

    let id_focus = id.clone();
    let id_click_focus = id.clone();
    let id_bar = id.clone();
    let id_content = id.clone();
    let id_drop = id.clone();
    let id_hover = id.clone();
    let ctx_hover = ctx.clone();
    let focus_behavior_hover = ctx.settings.focus_behavior_setting();
    let focus_behavior_click = focus_behavior_hover.clone();
    let focused_pane_attr = ctx.focused_pane;
    let id_focused_attr = id.clone();
    let focused_pane_frame = ctx.focused_pane;
    let zoomed_pane_frame = ctx.zoomed_pane;
    let id_focus_frame = id.clone();
    let id_zoom_frame = id.clone();

    // Does this pane hide its activity bar? (Role is stable for a pane's lifetime,
    // so evaluate the host predicate once, untracked.)
    let hide_bar = ctx
        .hide_activity_bar
        .as_ref()
        .map(|f| data.with_untracked(|d| f(d)))
        .unwrap_or(false);

    // Does this pane auto-hide its (kept) activity bar off the edge? Same one-shot
    // untracked evaluation, since it too is keyed on the pane's stable role.
    let auto_hide_bar = ctx
        .auto_hide_activity_bar
        .as_ref()
        .map(|f| data.with_untracked(|d| f(d)))
        .unwrap_or(false);

    // The pane axis follows the configured bar edge. Host-provided bottom-border
    // color (e.g. session color) remains reactive: the closure calls the host
    // fn, which can read live signals. `box-sizing:border-box` keeps the 2px
    // inside the pane.
    let id_border = id.clone();
    let border_fn = ctx.pane_border_color.clone();
    let edge = ctx.activity_bar_edge;
    let focus_bar_edge = (!hide_bar && !auto_hide_bar).then_some(edge);
    let focus_bar_width = ctx.activity_bar_style.width.clone();
    let pane_style = move || {
        let direction = if edge.is_horizontal() {
            "flex-direction:column;"
        } else {
            ""
        };
        let border = match border_fn.as_ref().and_then(|f| f(id_border.clone())) {
            Some(color) => {
                format!("box-sizing:border-box;border-bottom:2px solid {color};")
            }
            None => String::new(),
        };
        format!("{direction}{border}")
    };

    let focus_frame_style = move || {
        let focused = focused_pane_frame.get().as_ref() == Some(&id_focus_frame);
        let rendered_rect = if zoomed_pane_frame.get().as_ref() == Some(&id_zoom_frame) {
            Rect::FULL
        } else {
            rect.get()
        };
        focus_frame_css(focused, rendered_rect, focus_bar_edge, &focus_bar_width)
    };

    view! {
        <div
            class=PaneStyle::SCOPE
            data-mullion-focused=move || {
                if focused_pane_attr.get().as_ref() == Some(&id_focused_attr) {
                    "true"
                } else {
                    "false"
                }
            }
            node_ref=pane_ref
            style=pane_style
            on:mouseenter=move |_| {
                if focus_behavior_hover.get_untracked() == PaneFocusBehavior::Hover {
                    ctx_focus.focus_pane(&id_focus);
                }
            }
            on:mousedown=move |_| {
                if focus_behavior_click.get_untracked() == PaneFocusBehavior::Click {
                    ctx_click_focus.focus_pane(&id_click_focus);
                }
            }
        >
            {(!hide_bar).then(|| {
                let app_icon = ctx.app_icon.clone();
                if let Some(icon) = app_icon {
                    view! { <ActivityBar pane_id=id_bar.clone() data=data ctx=ctx.clone() app_icon=icon auto_hide=auto_hide_bar /> }.into_any()
                } else {
                    view! { <ActivityBar pane_id=id_bar.clone() data=data ctx=ctx.clone() auto_hide=auto_hide_bar /> }.into_any()
                }
            })}
            <div style="flex:1 1 0;min-width:0;min-height:0;overflow:hidden;position:relative">
                <PaneContent pane_id=id_content active_activity=active_activity data=data ctx=ctx.clone() />
                <DropOverlay pane_id=id_drop ctx=ctx />
                {hide_bar.then(|| view! { <PaneHoverControls pane_id=id_hover data=data ctx=ctx_hover /> })}
            </div>
            <div
                data-mullion-focus-frame=""
                aria-hidden="true"
                style=focus_frame_style
            />
        </div>
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct FocusEdges {
    top: bool,
    right: bool,
    bottom: bool,
    left: bool,
}

fn focus_edges(rect: Rect) -> FocusEdges {
    const ROOT_EDGE_EPSILON: f64 = 1e-7;
    FocusEdges {
        top: rect.top > ROOT_EDGE_EPSILON,
        right: rect.left + rect.width < 1.0 - ROOT_EDGE_EPSILON,
        bottom: rect.top + rect.height < 1.0 - ROOT_EDGE_EPSILON,
        left: rect.left > ROOT_EDGE_EPSILON,
    }
}

fn focus_frame_css(
    focused: bool,
    rect: Rect,
    activity_bar_edge: Option<ActivityBarEdge>,
    activity_bar_width: &str,
) -> String {
    let edges = focus_edges(rect);
    let width = |visible| {
        if visible {
            "var(--ml-focus-width, 2px)"
        } else {
            "0"
        }
    };
    let opacity = if focused { "1" } else { "0" };
    let (top, right, bottom, left) = match activity_bar_edge {
        Some(ActivityBarEdge::Left) => ("0", "0", "0", activity_bar_width),
        Some(ActivityBarEdge::Right) => ("0", activity_bar_width, "0", "0"),
        Some(ActivityBarEdge::Top) => (activity_bar_width, "0", "0", "0"),
        Some(ActivityBarEdge::Bottom) => ("0", "0", activity_bar_width, "0"),
        None => ("0", "0", "0", "0"),
    };
    format!(
        "position:absolute;top:{top};right:{right};bottom:{bottom};left:{left};z-index:6;\
         pointer-events:none;box-sizing:border-box;\
         border-style:solid;border-color:var(--ml-focus-color,var(--ml-primary,#00a4ef));\
         border-width:{} {} {} {};opacity:{opacity};transition:opacity 100ms ease-out;",
        width(edges.top),
        width(edges.right),
        width(edges.bottom),
        width(edges.left),
    )
}

#[cfg(test)]
mod focus_tests {
    use super::*;

    #[test]
    fn full_size_pane_has_no_internal_focus_edges() {
        assert_eq!(focus_edges(Rect::FULL), FocusEdges::default());
    }

    #[test]
    fn main_and_stack_layout_highlights_only_shared_edges() {
        let left = Rect {
            left: 0.0,
            top: 0.0,
            width: 0.4,
            height: 1.0,
        };
        let top_right = Rect {
            left: 0.4,
            top: 0.0,
            width: 0.6,
            height: 0.5,
        };
        let bottom_right = Rect {
            left: 0.4,
            top: 0.5,
            width: 0.6,
            height: 0.5,
        };

        assert_eq!(
            focus_edges(left),
            FocusEdges {
                right: true,
                ..FocusEdges::default()
            }
        );
        assert_eq!(
            focus_edges(top_right),
            FocusEdges {
                left: true,
                bottom: true,
                ..FocusEdges::default()
            }
        );
        assert_eq!(
            focus_edges(bottom_right),
            FocusEdges {
                left: true,
                top: true,
                ..FocusEdges::default()
            }
        );
    }

    #[test]
    fn focus_frame_uses_dedicated_theme_variables() {
        let css = focus_frame_css(
            true,
            Rect {
                left: 0.0,
                top: 0.0,
                width: 0.5,
                height: 1.0,
            },
            None,
            "28px",
        );
        assert!(css.contains("var(--ml-focus-color,var(--ml-primary,#00a4ef))"));
        assert!(css.contains("0 var(--ml-focus-width, 2px) 0 0"));
        assert!(css.contains("opacity:1"));
    }

    #[test]
    fn focus_frame_yields_to_a_pinned_activity_bar() {
        let rect = Rect {
            left: 0.4,
            top: 0.0,
            width: 0.6,
            height: 0.5,
        };

        let left = focus_frame_css(true, rect, Some(ActivityBarEdge::Left), "32px");
        assert!(left.contains("top:0;right:0;bottom:0;left:32px;z-index:6"));

        let top = focus_frame_css(true, rect, Some(ActivityBarEdge::Top), "30px");
        assert!(top.contains("top:30px;right:0;bottom:0;left:0;z-index:6"));
    }
}

const HC_SPLIT_H: &str = r#"<svg viewBox="0 0 16 16" width="13" height="13" fill="currentColor"><path d="M14 1H2a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V2a1 1 0 0 0-1-1zM2 2h5.5v12H2V2zm6.5 12V2H14v12H8.5z"/></svg>"#;
const HC_SPLIT_V: &str = r#"<svg viewBox="0 0 16 16" width="13" height="13" fill="currentColor"><path d="M14 1H2a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V2a1 1 0 0 0-1-1zM2 2h12v5.5H2V2zm0 6.5h12V14H2V8.5z"/></svg>"#;
const HC_CLOSE: &str = r#"<svg viewBox="0 0 16 16" width="13" height="13" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.707L8.707 8l3.647-3.646-.707-.708L8 7.293 4.354 3.646l-.707.708L7.293 8l-3.646 3.646.707.708L8 8.707z"/></svg>"#;

/// The management strip for a bar-less pane: split / close / drag-move, revealed
/// while the pane is focused. Keeps a hidden-bar pane fully manageable even
/// though it has no activity bar.
#[component]
fn PaneHoverControls<D: PaneData + Send + Sync>(
    pane_id: PaneId,
    data: Signal<D>,
    ctx: MullionContext<D>,
) -> impl IntoView {
    let focused = ctx.focused_pane;
    let pid_vis = pane_id.clone();
    let visible = Memo::new(move |_| focused.get().as_ref() == Some(&pid_vis));

    let ctx_h = ctx.clone();
    let ctx_v = ctx.clone();
    let ctx_c = ctx.clone();
    let ctx_d = ctx.clone();
    let ctx_de = ctx.clone();
    let pid_h = pane_id.clone();
    let pid_v = pane_id.clone();
    let pid_c = pane_id.clone();
    let pid_d = pane_id.clone();

    let wrap = move || {
        format!(
            "position:absolute;top:6px;right:6px;z-index:16;display:flex;gap:2px;padding:2px;\
             border-radius:6px;background:var(--ml-surface);border:1px solid var(--ml-border);\
             opacity:{};pointer-events:{};transition:opacity .12s;",
            if visible.get() { "0.95" } else { "0" },
            if visible.get() { "auto" } else { "none" },
        )
    };
    let btn = "display:flex;align-items:center;justify-content:center;width:22px;height:22px;\
               padding:0;border:none;background:transparent;color:var(--ml-text);cursor:pointer;\
               border-radius:4px;opacity:0.75;";
    let fresh_id = || PaneId::new(format!("{:.0}", web_sys::js_sys::Math::random() * 1e12));

    view! {
        <div style=wrap>
            <div title="Move pane" draggable="true" style=format!("{btn}cursor:grab;font-size:13px;")
                on:dragstart=move |ev| {
                    ctx_d.drag.set(Some(DragPayload::Pane(pid_d.clone())));
                    if let Some(dt) = ev.data_transfer() {
                        let _ = dt.set_data("text/plain", &pid_d.0);
                        dt.set_effect_allowed("move");
                    }
                }
                on:dragend=move |_| ctx_de.drag.set(None)
            >"⠿"</div>
            <button title="Split horizontal" style=btn inner_html=HC_SPLIT_H
                on:click=move |_| { ctx_h.split_pane(&pid_h, SplitDirection::Horizontal, fresh_id(), data.get_untracked()); } />
            <button title="Split vertical" style=btn inner_html=HC_SPLIT_V
                on:click=move |_| { ctx_v.split_pane(&pid_v, SplitDirection::Vertical, fresh_id(), data.get_untracked()); } />
            <button title="Close pane" style=btn inner_html=HC_CLOSE
                on:click=move |_| { ctx_c.close_pane(&pid_c); } />
        </div>
    }
}

#[component]
fn SplitHandleSlot<D: PaneData + Send + Sync>(
    split_key: PaneId,
    ctx: MullionContext<D>,
    container_ref: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    // Direction is read reactively from the tree so that
    // `change_split_direction` updates an existing handle in place instead
    // of requiring a remount (the `<For>` key is just `split_key`).
    let key_dir = split_key.clone();
    let ctx_dir = ctx.clone();
    let direction_memo = Memo::new(move |prev: Option<&SplitDirection>| {
        ctx_dir
            .tree
            .with(|t| find_split_direction(t, &key_dir))
            .or_else(|| prev.copied())
            .unwrap_or(SplitDirection::Horizontal)
    });

    // Parent rect of the split (reactive — changes when ancestor ratios change).
    let key_rect = split_key.clone();
    let ctx_rect = ctx.clone();
    let parent_rect = Memo::new(move |prev: Option<&Rect>| {
        ctx_rect.tree.with(|tree| {
            let ctx_for_ratio = ctx_rect.clone();
            split_parent_rect(tree, &key_rect, move |key| {
                ctx_for_ratio.ratio_signal(key).get()
            })
            .unwrap_or_else(|| prev.copied().unwrap_or(Rect::FULL))
        })
    });

    let ratio_sig = ctx.ratio_signal(&split_key);

    // Positioning is inline (absolute, derived from the split's parent rect
    // and its ratio). All other visuals — cursor, hit-target thickness, bar
    // thickness, bar color, hover color — come from `SplitHandleStyle` so
    // consumers keep their existing theming surface.
    //
    // `--msh-target-thickness` is declared by `SplitHandleStyle` on the
    // `.msh` scope, which is applied via the class below. The `calc()`
    // recentres the handle on the split boundary.
    let ratio_for_style = ratio_sig.clone();
    let handle_style = move || {
        let r = parent_rect.get();
        let ratio = ratio_for_style.get();
        match direction_memo.get() {
            SplitDirection::Horizontal => {
                let x_pct = (r.left + r.width * ratio) * 100.0;
                let y_pct = r.top * 100.0;
                let h_pct = r.height * 100.0;
                format!(
                    "position:absolute;z-index:5;\
                     left:calc({x_pct}% - var(--msh-target-thickness) / 2);\
                     top:{y_pct}%;height:{h_pct}%;",
                )
            }
            SplitDirection::Vertical => {
                let y_pct = (r.top + r.height * ratio) * 100.0;
                let x_pct = r.left * 100.0;
                let w_pct = r.width * 100.0;
                format!(
                    "position:absolute;z-index:5;\
                     top:calc({y_pct}% - var(--msh-target-thickness) / 2);\
                     left:{x_pct}%;width:{w_pct}%;",
                )
            }
        }
    };

    // Class: base SplitHandleStyle scope + axis modifier + DRAGGING
    // modifier driven by leptos-resize's drag signal so the bar stays
    // highlighted for the full duration of a drag (even when the
    // cursor leaves the handle bounds).
    let dragging = RwSignal::new(false);
    let handle_class = move || {
        let mods = if dragging.get() {
            vec![
                match direction_memo.get() {
                    SplitDirection::Horizontal => SplitHandleModifier::Horizontal,
                    SplitDirection::Vertical => SplitHandleModifier::Vertical,
                },
                SplitHandleModifier::Dragging,
            ]
        } else {
            vec![match direction_memo.get() {
                SplitDirection::Horizontal => SplitHandleModifier::Horizontal,
                SplitDirection::Vertical => SplitHandleModifier::Vertical,
            }]
        };
        SplitHandleStyle::class(&mods)
    };

    // Drag — delegate the cross-cutting parts (document listeners,
    // global cursor lock, dragging-class toggle) to leptos-resize's
    // use_drag hook. on_move recomputes the split ratio from the
    // pixel delta + the split's parent_rect (normalized in [0,1]
    // relative to the root container).
    let ctx_drag = ctx.clone();
    let key_drag = split_key.clone();
    let start_ratio = RwSignal::new(0.5_f64);
    let ratio_for_start = ratio_sig.clone();
    let on_start = Callback::new(move |_| {
        start_ratio.set(ratio_for_start.get_untracked());
    });
    let ctx_move = ctx_drag.clone();
    let key_move = key_drag.clone();
    let direction_move = direction_memo;
    let container_for_move = container_ref;
    let on_move = Callback::new(move |delta_px: f64| {
        let Some(container) = container_for_move.get_untracked() else {
            return;
        };
        let container: &web_sys::HtmlElement = container.as_ref();
        let root_rect = container.get_bounding_client_rect();
        let root_dim = match direction_move.get_untracked() {
            SplitDirection::Horizontal => root_rect.width(),
            SplitDirection::Vertical => root_rect.height(),
        };
        if root_dim <= 0.0 {
            return;
        }
        let parent = parent_rect.get_untracked();
        let parent_dim = match direction_move.get_untracked() {
            SplitDirection::Horizontal => parent.width,
            SplitDirection::Vertical => parent.height,
        };
        if parent_dim <= 0.0 {
            return;
        }
        let delta_ratio = delta_px / (root_dim * parent_dim);
        let new_ratio = start_ratio.get_untracked() + delta_ratio;
        ctx_move.resize_split(&key_move, new_ratio);
    });

    let direction_for_drag = direction_memo;
    let on_mousedown = move |ev: web_sys::MouseEvent| {
        let dir = match direction_for_drag.get_untracked() {
            SplitDirection::Horizontal => LrhDirection::Horizontal,
            SplitDirection::Vertical => LrhDirection::Vertical,
        };
        let handler = use_drag(dragging, dir, Some(on_start), Some(on_move), None);
        handler(ev);
    };

    view! {
        <div class=handle_class style=handle_style on:mousedown=on_mousedown>
            <span class=SplitHandleStyle::BAR />
        </div>
    }
}
