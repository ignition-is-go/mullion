use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::context::MullionContext;
use crate::theme::DropOverlayStyle;
use crate::tree::{DropEdge, PaneData, PaneId};

/// Overlay that appears on a pane during drag operations.
/// Detects which edge the cursor is over and highlights it.
/// On drop, calls `ctx.move_pane`.
#[component]
pub fn DropOverlay<D: PaneData + Send + Sync>(
    pane_id: PaneId,
    ctx: MullionContext<D>,
) -> impl IntoView {
    let (hover_edge, set_hover_edge) = signal(Option::<DropEdge>::None);
    let dragging = ctx.dragging_pane;

    // These are non-reactive — rendered once when the overlay is visible.
    // We build the overlay outside the reactive closure so event handlers are Fn, not FnOnce.
    let overlay_view = {
        let ctx_drop = ctx.clone();
        let pane_id_drop = pane_id.clone();

        let on_dragover = move |ev: web_sys::DragEvent| {
            ev.prevent_default();
            if let Some(dt) = ev.data_transfer() {
                dt.set_drop_effect("move");
            }

            let target = ev.current_target().unwrap();
            let el: web_sys::HtmlElement = target.unchecked_into();
            let rect = el.get_bounding_client_rect();

            let x = ev.client_x() as f64 - rect.left();
            let y = ev.client_y() as f64 - rect.top();
            let w = rect.width();
            let h = rect.height();

            let nx = x / w;
            let ny = y / h;

            let edge = if nx < 0.25 {
                DropEdge::Left
            } else if nx > 0.75 {
                DropEdge::Right
            } else if ny < 0.25 {
                DropEdge::Top
            } else if ny > 0.75 {
                DropEdge::Bottom
            } else {
                DropEdge::Center
            };

            set_hover_edge.set(Some(edge));
        };

        let on_dragleave = move |_: web_sys::DragEvent| {
            set_hover_edge.set(None);
        };

        let on_drop = move |ev: web_sys::DragEvent| {
            ev.prevent_default();
            let edge = hover_edge.get_untracked();
            let source = dragging.get_untracked();
            set_hover_edge.set(None);
            ctx_drop.dragging_pane.set(None);

            if let (Some(source_id), Some(edge)) = (source, edge) {
                if source_id != pane_id_drop {
                    ctx_drop.move_pane(&source_id, &pane_id_drop, edge);
                }
            }
        };

        view! {
            <div style="position:absolute;inset:0;z-index:20"
                 on:dragover=on_dragover
                 on:dragleave=on_dragleave
                 on:drop=on_drop>
                {move || {
                    hover_edge.get().map(|e| {
                        let style = edge_indicator_style(e);
                        view! { <div class=DropOverlayStyle::SCOPE style={style}></div> }
                    })
                }}
            </div>
        }
    };

    view! {
        {move || {
            let is_dragging = dragging.get().is_some();
            let is_self = dragging.get().as_ref() == Some(&pane_id);

            if is_dragging && !is_self {
                Some(overlay_view.clone())
            } else {
                None
            }
        }}
    }
}

fn edge_indicator_style(edge: DropEdge) -> String {
    let base = "transition:all 0.1s ease";
    match edge {
        DropEdge::Left => format!("{};left:0;top:0;bottom:0;width:50%", base),
        DropEdge::Right => format!("{};right:0;top:0;bottom:0;width:50%", base),
        DropEdge::Top => format!("{};left:0;right:0;top:0;height:50%", base),
        DropEdge::Bottom => format!("{};left:0;right:0;bottom:0;height:50%", base),
        DropEdge::Center => format!("{};inset:0", base),
    }
}
