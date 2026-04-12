use leptos::prelude::*;

use crate::context::MullionContext;
use crate::tree::{PaneData, PaneNode, SplitDirection};

use super::activity_bar::ActivityBar;
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
            let pane_theme = &ctx.pane_theme;
            let (data_read, _data_write) = signal(data);

            let pane_style = format!(
                "display:flex;flex-direction:row;width:100%;height:100%;overflow:hidden;background:{};color:{}",
                pane_theme.background, pane_theme.color
            );

            let ctx_focus = ctx.clone();
            view! {
                <div style={pane_style}
                     on:mouseenter=move |_| { ctx_focus.focused_pane.set(Some(id)); }>
                    <ActivityBar pane_id=id data=data_read ctx=ctx.clone() />
                    <div style="flex:1;overflow:hidden;position:relative">
                        <PaneContent pane_id=id activity=active_activity data=data_read ctx=ctx />
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
                        let handle_theme = ctx.split_handle_theme.clone();
                        view! {
                            <SplitHandle
                                direction=direction
                                theme=handle_theme
                                on_resize=Callback::new(move |ratio: f64| {
                                    ctx.resize_pane(leaf_id, ratio);
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
