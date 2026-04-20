use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MouseEvent;

use crate::context::MullionContext;
use crate::theme::MullionTheme;
use crate::tree::{
    collect_split_keys, find_split_direction, leaf_rect, split_parent_rect, ActivityId,
    PaneData, PaneId, PaneNode, Rect, SplitDirection,
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
                width: 100%;
                height: 100%;
                overflow: hidden;
            }
        })
    }
}

use super::activity_bar::ActivityBar;
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
pub fn PaneView<D: PaneData + Send + Sync>(
    ctx: MullionContext<D>,
) -> impl IntoView {
    let ctx_leaves = ctx.clone();
    let leaves = Memo::new(move |_| ctx_leaves.tree.with(|t| t.leaf_ids()));

    let ctx_splits = ctx.clone();
    let splits = Memo::new(move |_| ctx_splits.tree.with(|t| collect_split_keys(t)));

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
fn LeafSlot<D: PaneData + Send + Sync>(
    id: PaneId,
    ctx: MullionContext<D>,
) -> impl IntoView {
    let id_rect = id.clone();
    let ctx_rect = ctx.clone();
    let rect = Memo::new(move |prev: Option<&Rect>| {
        ctx_rect.tree.with(|tree| {
            let ctx_for_ratio = ctx_rect.clone();
            leaf_rect(tree, &id_rect, move |key| ctx_for_ratio.ratio_signal(key).get())
                .unwrap_or_else(|| prev.copied().unwrap_or(Rect::FULL))
        })
    });

    let slot_style = move || {
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
            <LeafView id=id ctx=ctx />
        </div>
    }
}

#[component]
fn LeafView<D: PaneData + Send + Sync>(
    id: PaneId,
    ctx: MullionContext<D>,
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
            Some(PaneNode::Leaf { active_activity, .. }) => active_activity.clone(),
            _ => prev.cloned().unwrap_or(None),
        })
    });
    let active_activity: Signal<Option<ActivityId>> = activity_memo.into();

    let ctx_focus = ctx.clone();
    let ctx_ref = ctx.clone();
    let pane_ref: NodeRef<leptos::html::Div> = NodeRef::new();

    let id_for_ref = id.clone();
    pane_ref.on_load(move |el| {
        let html_el: web_sys::HtmlElement = el.into();
        ctx_ref.register_pane_element(id_for_ref.clone(), html_el);
    });

    let id_focus = id.clone();
    let id_bar = id.clone();
    let id_content = id.clone();
    let id_drop = id.clone();
    view! {
        <div
            class=PaneStyle::SCOPE
            node_ref=pane_ref
            on:mouseenter=move |_| { ctx_focus.focused_pane.set(Some(id_focus.clone())); }
        >
            {
                let app_icon = ctx.app_icon.clone();
                if let Some(icon) = app_icon {
                    view! { <ActivityBar pane_id=id_bar.clone() data=data ctx=ctx.clone() app_icon=icon /> }.into_any()
                } else {
                    view! { <ActivityBar pane_id=id_bar.clone() data=data ctx=ctx.clone() /> }.into_any()
                }
            }
            <div style="flex:1;overflow:hidden;position:relative">
                <PaneContent pane_id=id_content active_activity=active_activity data=data ctx=ctx.clone() />
                <DropOverlay pane_id=id_drop ctx=ctx />
            </div>
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
            split_parent_rect(tree, &key_rect, move |key| ctx_for_ratio.ratio_signal(key).get())
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

    let handle_class = move || {
        SplitHandleStyle::class(&[match direction_memo.get() {
            SplitDirection::Horizontal => SplitHandleModifier::Horizontal,
            SplitDirection::Vertical => SplitHandleModifier::Vertical,
        }])
    };

    // Drag: mousedown starts a document-level mousemove/mouseup loop.
    // Each mousemove reads the split's current parent rect and direction
    // (untracked, so the drag does not establish reactive dependencies)
    // and converts the mouse position within the root container into a
    // new ratio for this split. `resize_split` clamps and emits the
    // Resized event.
    let ctx_drag = ctx.clone();
    let key_drag = split_key.clone();
    let on_mousedown = move |ev: MouseEvent| {
        ev.prevent_default();
        let container: web_sys::HtmlElement = match container_ref.get() {
            Some(el) => el.into(),
            None => return,
        };
        let document = web_sys::window().unwrap().document().unwrap();

        let closures: Rc<RefCell<Option<(
            Closure<dyn FnMut(MouseEvent)>,
            Closure<dyn FnMut(MouseEvent)>,
        )>>> = Rc::new(RefCell::new(None));
        let closures_for_up = closures.clone();
        let doc_for_up = document.clone();

        let ctx_move = ctx_drag.clone();
        let key_move = key_drag.clone();
        let parent_rect_move = parent_rect;
        let direction_move = direction_memo;
        let container_for_move = container.clone();
        let mousemove_cb = Closure::<dyn FnMut(MouseEvent)>::new(move |ev: MouseEvent| {
            let root_rect = container_for_move.get_bounding_client_rect();
            if root_rect.width() <= 0.0 || root_rect.height() <= 0.0 {
                return;
            }
            let parent = parent_rect_move.get_untracked();
            let ratio = match direction_move.get_untracked() {
                SplitDirection::Horizontal => {
                    let x_frac = (ev.client_x() as f64 - root_rect.left()) / root_rect.width();
                    (x_frac - parent.left) / parent.width
                }
                SplitDirection::Vertical => {
                    let y_frac = (ev.client_y() as f64 - root_rect.top()) / root_rect.height();
                    (y_frac - parent.top) / parent.height
                }
            };
            ctx_move.resize_split(&key_move, ratio);
        });

        let mouseup_cb = Closure::<dyn FnMut(MouseEvent)>::new(move |_: MouseEvent| {
            if let Some((ref move_cb, ref up_cb)) = *closures_for_up.borrow() {
                let _ = doc_for_up.remove_event_listener_with_callback(
                    "mousemove",
                    move_cb.as_ref().unchecked_ref(),
                );
                let _ = doc_for_up.remove_event_listener_with_callback(
                    "mouseup",
                    up_cb.as_ref().unchecked_ref(),
                );
            }
            closures_for_up.borrow_mut().take();
        });

        document
            .add_event_listener_with_callback("mousemove", mousemove_cb.as_ref().unchecked_ref())
            .unwrap();
        document
            .add_event_listener_with_callback("mouseup", mouseup_cb.as_ref().unchecked_ref())
            .unwrap();

        *closures.borrow_mut() = Some((mousemove_cb, mouseup_cb));
    };

    view! {
        <div class=handle_class style=handle_style on:mousedown=on_mousedown>
            <span class=SplitHandleStyle::BAR />
        </div>
    }
}
