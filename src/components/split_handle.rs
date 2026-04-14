use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::MouseEvent;

use crate::theme::{SplitHandleModifier, SplitHandleStyle};
use crate::tree::SplitDirection;

/// A draggable handle between two split panes for resizing.
///
/// Renders a wider invisible hover target with a narrower visible separator
/// centered inside it. All styling is driven by `SplitHandleStyle` via
/// css-styled scoped CSS classes and custom properties.
#[component]
pub fn SplitHandle(
    direction: SplitDirection,
    on_resize: Callback<f64>,
) -> impl IntoView {
    let modifier = match direction {
        SplitDirection::Horizontal => SplitHandleModifier::Horizontal,
        SplitDirection::Vertical => SplitHandleModifier::Vertical,
    };

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
            class=SplitHandleStyle::class(&[modifier])
            on:mousedown=on_mousedown
        >
            <div class=SplitHandleStyle::BAR />
        </div>
    }
}
