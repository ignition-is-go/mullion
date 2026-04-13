use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MouseEvent;

use crate::theme::SplitHandleTheme;
use crate::tree::SplitDirection;

/// CSS class for the split handle hover/drag target area.
pub const CLASS_SPLIT_TARGET: &str = "msh-target";
/// CSS class for the visible split handle bar.
pub const CLASS_SPLIT_BAR: &str = "msh-bar";

/// A draggable handle between two split panes for resizing.
///
/// Renders a wider invisible hover target with a narrower visible separator
/// centered inside it. The hover target thickness and visible thickness are
/// independently configurable via `SplitHandleTheme`.
///
/// Hover styles are injected once at the root level (`MullionRoot` /
/// `MullionProvider` / `MullionPaneTree`), not per-instance.
#[component]
pub fn SplitHandle(
    direction: SplitDirection,
    on_resize: Callback<f64>,
    #[prop(optional)]
    theme: Option<SplitHandleTheme>,
) -> impl IntoView {
    let theme = theme.unwrap_or_default();

    let cursor = match direction {
        SplitDirection::Horizontal => "col-resize",
        SplitDirection::Vertical => "row-resize",
    };

    let (target_dim, bar_dim) = match direction {
        SplitDirection::Horizontal => (
            format!("width:{t};min-width:{t}", t = theme.hover_target_thickness),
            format!("width:{t};height:100%", t = theme.thickness),
        ),
        SplitDirection::Vertical => (
            format!("height:{t};min-height:{t}", t = theme.hover_target_thickness),
            format!("height:{t};width:100%", t = theme.thickness),
        ),
    };

    let target_style = format!(
        "{target_dim};cursor:{cursor};display:flex;align-items:center;justify-content:center;flex-shrink:0"
    );

    let bar_style = format!(
        "{bar_dim};background:{c};transition:background 0.1s ease;pointer-events:none",
        c = theme.color,
    );

    let on_mousedown = move |ev: MouseEvent| {
        ev.prevent_default();

        let target = ev.current_target().unwrap();
        let handle_el: web_sys::HtmlElement = target.unchecked_into();
        let parent = handle_el
            .parent_element()
            .expect("split handle must have a parent");

        let dir = direction;
        let document = web_sys::window().unwrap().document().unwrap();

        // Get sibling elements for live CSS updates during drag
        let children = parent.children();
        let first_child: Option<web_sys::HtmlElement> = children
            .item(0)
            .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok());
        let last_child: Option<web_sys::HtmlElement> = children
            .item(children.length() - 1)
            .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok());

        let final_ratio: Rc<RefCell<f64>> = Rc::new(RefCell::new(0.5));

        let closures: Rc<RefCell<Option<(
            Closure<dyn FnMut(MouseEvent)>,
            Closure<dyn FnMut(MouseEvent)>,
        )>>> = Rc::new(RefCell::new(None));

        let closures_for_up = closures.clone();
        let doc_for_up = document.clone();
        let ratio_for_move = final_ratio.clone();

        let mousemove_cb = Closure::<dyn FnMut(MouseEvent)>::new(move |ev: MouseEvent| {
            let rect = parent.get_bounding_client_rect();
            let ratio = match dir {
                SplitDirection::Horizontal => {
                    (ev.client_x() as f64 - rect.left()) / rect.width()
                }
                SplitDirection::Vertical => {
                    (ev.client_y() as f64 - rect.top()) / rect.height()
                }
            };
            let ratio = ratio.clamp(0.1, 0.9);
            *ratio_for_move.borrow_mut() = ratio;

            // Update CSS directly for smooth dragging without re-rendering
            let first_pct = format!("{}%", ratio * 100.0);
            let second_pct = format!("{}%", (1.0 - ratio) * 100.0);
            if let Some(ref el) = first_child {
                let _ = el.style().set_property("flex-basis", &first_pct);
            }
            if let Some(ref el) = last_child {
                let _ = el.style().set_property("flex-basis", &second_pct);
            }
        });

        let mouseup_cb = Closure::<dyn FnMut(MouseEvent)>::new(move |_: MouseEvent| {
            // Remove listeners
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

            // Commit the final ratio to the tree (triggers re-render once)
            let ratio = *final_ratio.borrow();
            on_resize.run(ratio);
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
        <div
            class=CLASS_SPLIT_TARGET
            style={target_style}
            on:mousedown=on_mousedown
        >
            <div class=CLASS_SPLIT_BAR style={bar_style} />
        </div>
    }
}
