use leptos::prelude::*;

use crate::context::MullionContext;
use crate::theme::PaneStyle;
use crate::tree::{PaneData, PaneNode, SplitDirection};

use super::activity_bar::ActivityBar;
use super::drop_overlay::DropOverlay;
use super::pane_content::PaneContent;
use super::split_handle::SplitHandle;

/// Recursively renders a pane tree node.
#[component]
pub fn PaneView<D: PaneData + Send + Sync>(
    node: PaneNode<D>,
    ctx: MullionContext<D>,
) -> impl IntoView {
    match node {
        PaneNode::Leaf {
            id,
            active_activity,
            data,
        } => {
            let (data_read, _data_write) = signal(data);

            let ctx_focus = ctx.clone();
            let ctx_ref = ctx.clone();
            let pane_ref: NodeRef<leptos::html::Div> = NodeRef::new();

            // Register the DOM element once mounted
            let id_for_ref = id.clone();
            Effect::new(move |_| {
                if let Some(el) = pane_ref.get() {
                    let html_el: web_sys::HtmlElement = el.into();
                    ctx_ref.register_pane_element(id_for_ref.clone(), html_el);
                }
            });

            let id_focus = id.clone();
            let id_bar = id.clone();
            let id_content = id.clone();
            let id_drop = id.clone();
            view! {
                <div class=PaneStyle::SCOPE
                     node_ref=pane_ref
                     on:mouseenter=move |_| { ctx_focus.focused_pane.set(Some(id_focus.clone())); }>
                    {
                        let app_icon = ctx.app_icon.clone();
                        if let Some(icon) = app_icon {
                            view! { <ActivityBar pane_id=id_bar.clone() data=data_read ctx=ctx.clone() app_icon=icon /> }.into_any()
                        } else {
                            view! { <ActivityBar pane_id=id_bar.clone() data=data_read ctx=ctx.clone() /> }.into_any()
                        }
                    }
                    <div style="flex:1;overflow:hidden;position:relative">
                        <PaneContent pane_id=id_content activity=active_activity data=data_read ctx=ctx.clone() />
                        <DropOverlay pane_id=id_drop ctx=ctx />
                    </div>
                </div>
            }
            .into_any()
        }
        PaneNode::Split {
            direction,
            ratio,
            first,
            second,
        } => {
            let first_pct = format!("{}%", ratio * 100.0);
            let second_pct = format!("{}%", (1.0 - ratio) * 100.0);

            let flex_dir = match direction {
                SplitDirection::Horizontal => "row",
                SplitDirection::Vertical => "column",
            };

            let container_style = format!(
                "display:flex;flex-direction:{};width:100%;height:100%",
                flex_dir
            );

            let first_style = format!(
                "flex-basis:{};flex-shrink:1;flex-grow:0;min-width:0;min-height:0;overflow:hidden",
                first_pct
            );
            let second_style = format!(
                "flex-basis:{};flex-shrink:1;flex-grow:0;min-width:0;min-height:0;overflow:hidden",
                second_pct
            );

            let first_leaf_id = first.leaf_ids().into_iter().next();

            view! {
                <div style={container_style}>
                    <div style={first_style}>
                        <PaneView node=*first ctx=ctx.clone() />
                    </div>
                    {first_leaf_id.map(|leaf_id| {
                        let ctx = ctx.clone();
                        view! {
                            <SplitHandle
                                direction=direction
                                on_resize=Callback::new(move |ratio: f64| {
                                    ctx.resize_pane(&leaf_id, ratio);
                                })
                            />
                        }
                    })}
                    <div style={second_style}>
                        <PaneView node=*second ctx=ctx />
                    </div>
                </div>
            }
            .into_any()
        }
    }
}
