use std::cell::Cell;
use std::rc::Rc;

use leptos::prelude::*;
use leptos_resize::{use_drag, Direction as LrhDirection};
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;

use crate::theme::MullionTheme;
use crate::tree::SplitDirection;

/// Style for split handles, powered by css-styled.
///
/// Generates scoped CSS with typed class names, CSS custom properties,
/// and spec-validated base CSS.
#[derive(css_styled::StyledComponent, Clone, Debug)]
#[component(scope = "msh")]
#[component(theme = MullionTheme)]
#[component(class(bar = "msh-bar"))]
#[component(modifier(horizontal, vertical, dragging))]
#[component(base_css)]
pub struct SplitHandleStyle {
    #[prop(var = "--msh-thickness", default = "4px")]
    pub thickness: String,
    #[prop(var = "--msh-target-thickness", default = "8px")]
    pub hover_target_thickness: String,
    #[prop(var = "--msh-color", default = theme.border)]
    pub color: String,
    /// Bar color while hovered OR actively dragging. Drag state is
    /// driven by leptos-resize via the DRAGGING modifier toggled on
    /// the scope element.
    #[prop(var = "--msh-hover-color", default = theme.highlight)]
    pub hover_color: String,
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
            /* Bar has pointer-events:none, so :hover never fires on
               the bar itself — hover the SCOPE wrapper (the actual
               mouse hit-target with cursor:resize) and propagate
               down. Same selector shape for the DRAGGING modifier so
               the highlight is identical in both states. */
            SCOPE:hover BAR {
                background: var(--msh-hover-color);
            }
            SCOPE.DRAGGING BAR {
                background: var(--msh-hover-color);
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

/// A draggable handle between two split panes for resizing.
///
/// Renders a wider invisible hover target with a narrower visible
/// separator centered inside it. Drag tracking, the dragging-class
/// toggle, and the global cursor lock are all delegated to
/// `leptos_resize::use_drag`. The handle snapshots the starting
/// flex-basis of its two sibling panes on drag start and updates
/// their inline style directly during drag (smooth resize without
/// re-rendering), then commits the final ratio via `on_resize` on
/// release.
#[component]
pub fn SplitHandle(
    direction: SplitDirection,
    on_resize: Callback<f64>,
) -> impl IntoView {
    let axis_modifier = match direction {
        SplitDirection::Horizontal => SplitHandleModifier::Horizontal,
        SplitDirection::Vertical => SplitHandleModifier::Vertical,
    };

    // Per-drag snapshot: (start_ratio, parent_dim_px, first_child, last_child).
    // SendWrapper because Callbacks require Send+Sync but the inner
    // Rc<Cell<_>> + HtmlElements are wasm-only !Send/!Sync types.
    type DragState = (f64, f64, web_sys::HtmlElement, web_sys::HtmlElement);
    let state: SendWrapper<Rc<Cell<Option<DragState>>>> =
        SendWrapper::new(Rc::new(Cell::new(None)));

    // NodeRef on the outer div so on_start can walk to its parent and
    // discover the two sibling panes.
    let handle_ref: NodeRef<leptos::html::Div> = NodeRef::new();

    let state_start = state.clone();
    let on_start = Callback::new(move |_| {
        let Some(el) = handle_ref.get_untracked() else {
            return;
        };
        let el: &web_sys::HtmlElement = el.as_ref();
        let Some(parent) = el.parent_element() else {
            return;
        };
        let children = parent.children();
        let first: Option<web_sys::HtmlElement> = children
            .item(0)
            .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok());
        let last: Option<web_sys::HtmlElement> = children
            .item(children.length().saturating_sub(1))
            .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok());
        let (Some(first), Some(last)) = (first, last) else {
            return;
        };
        let parent_rect = parent.get_bounding_client_rect();
        let first_rect = first.get_bounding_client_rect();
        let (parent_dim, first_dim) = match direction {
            SplitDirection::Horizontal => (parent_rect.width(), first_rect.width()),
            SplitDirection::Vertical => (parent_rect.height(), first_rect.height()),
        };
        if parent_dim <= 0.0 {
            return;
        }
        let start_ratio = (first_dim / parent_dim).clamp(0.0, 1.0);
        (*state_start).set(Some((start_ratio, parent_dim, first, last)));
    });

    let state_move = state.clone();
    let on_move = Callback::new(move |delta: f64| {
        let Some((start_ratio, parent_dim, first, last)) = (*state_move).take() else {
            return;
        };
        let new_ratio = (start_ratio + delta / parent_dim).clamp(0.1, 0.9);
        let first_pct = format!("{}%", new_ratio * 100.0);
        let second_pct = format!("{}%", (1.0 - new_ratio) * 100.0);
        let _ = first.style().set_property("flex-basis", &first_pct);
        let _ = last.style().set_property("flex-basis", &second_pct);
        (*state_move).set(Some((start_ratio, parent_dim, first, last)));
    });

    let state_end = state.clone();
    let on_end = Callback::new(move |delta: f64| {
        let Some((start_ratio, parent_dim, _first, _last)) = (*state_end).take() else {
            return;
        };
        let final_ratio = (start_ratio + delta / parent_dim).clamp(0.1, 0.9);
        on_resize.run(final_ratio);
    });

    let dragging = RwSignal::new(false);
    let lrh_direction = match direction {
        SplitDirection::Horizontal => LrhDirection::Horizontal,
        SplitDirection::Vertical => LrhDirection::Vertical,
    };
    let on_mousedown = use_drag(
        dragging,
        lrh_direction,
        Some(on_start),
        Some(on_move),
        Some(on_end),
    );

    let class_sig = move || {
        if dragging.get() {
            SplitHandleStyle::class(&[axis_modifier, SplitHandleModifier::Dragging])
        } else {
            SplitHandleStyle::class(&[axis_modifier])
        }
    };

    view! {
        <div
            class=class_sig
            node_ref=handle_ref
            on:mousedown=on_mousedown
        >
            <div class=SplitHandleStyle::BAR />
        </div>
    }
}
