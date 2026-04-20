use leptos::prelude::*;

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

    view! {
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
    }
}
