use leptos::prelude::*;

use super::pane_header::PaneHeader;
use crate::context::MullionContext;
use crate::tree::{ActivityId, PaneData, PaneId};

/// Renders the content area for a pane by delegating to the active activity's
/// render function.
///
/// Subscribes reactively to `data` and `active_activity` — changes to other
/// leaves' state do not re-render this component.
#[component]
pub fn PaneContent<D: PaneData + Send + Sync>(
    pane_id: PaneId,
    active_activity: Signal<Option<ActivityId>>,
    data: Signal<D>,
    ctx: MullionContext<D>,
) -> impl IntoView {
    let ctx_for_memo = ctx.clone();
    let resolved_activity = Memo::new(move |_| {
        let d = data.get();
        let active = active_activity.get();
        if let Some(ref act_id) = active {
            let available = ctx_for_memo.activities_for_pane(&d);
            if available.iter().any(|a| a.def.id == *act_id) {
                return Some(act_id.clone());
            }
        }
        let available = ctx_for_memo.activities_for_pane(&d);
        available.first().map(|a| a.def.id.clone())
    });

    // Flex column: [header band (auto height)] + [activity body].
    //
    // The whole column is positioned `absolute; inset:0` so it fills the
    // (position:relative) leaf slot with a definite size — sidestepping the
    // "percentage height collapses to 0 in a flex child" trap. The body is a
    // `flex:1 1 0; min-height:0` child and is itself `position:relative`, so
    // an activity that paints with `position:absolute; inset:0` fills exactly
    // the area below the header.
    //
    // `isolation:isolate` makes this column its own stacking context, which is
    // the boundary between activity content and pane chrome: whatever z-index
    // an activity uses is resolved *inside* this box and can never compete with
    // the chrome band (see [`crate::components::overlay`]). Note the isolate
    // wraps the content column only — `DropOverlay` is deliberately a sibling
    // outside it in `PaneView`, so drag chrome still paints above content.
    //
    // Consequence for activities: content that must escape the pane cannot do
    // it with a bigger number any more. It has to leave the pane properly, via
    // [`crate::MullionOverlay`].
    let pid_header = pane_id.clone();
    let ctx_header = ctx.clone();
    let show_header = ctx.show_pane_header;
    view! {
        <div style="position:absolute;inset:0;display:flex;flex-direction:column;overflow:hidden;isolation:isolate">
            {show_header.then(|| view! {
                <PaneHeader
                    pane_id=pid_header
                    resolved=resolved_activity.into()
                    data=data
                    ctx=ctx_header
                />
            })}
            <div style="flex:1 1 0;position:relative;min-height:0;overflow:hidden">
                {
                    let pane_id = pane_id.clone();
                    move || {
                    let act_id = resolved_activity.get();
                    match act_id {
                        Some(id) => {
                            let render_fn = ctx.activities.with_value(|acts| {
                                acts.iter().find(|a| a.def.id == id).map(|a| a.def.render)
                            });
                            match render_fn {
                                Some(render) => render(pane_id.clone(), data),
                                None => view! { <div>"Activity not found"</div> }.into_any(),
                            }
                        }
                        None => view! { <div></div> }.into_any(),
                    }
                }}
            </div>
        </div>
    }
}
