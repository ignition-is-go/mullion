//! Mullion-owned overlay layer for content that must escape its pane.
//!
//! # The chrome z-band
//!
//! Mullion paints its own chrome in a deliberately small band, all of it inside
//! the pane tree:
//!
//! | z-index | element                                    |
//! |---------|--------------------------------------------|
//! | 5       | split handles ([`super::pane_view`])       |
//! | 10      | activity bar panel ([`super::activity_bar`]) |
//! | 20      | drop overlay ([`super::drop_overlay`])     |
//!
//! Activity content is confined below that band. Anything an activity needs to
//! paint *above* it — a full-screen modal, a command palette — must leave the
//! pane entirely rather than try to out-bid chrome with a bigger number.
//! [`MullionOverlay`] is that exit.
//!
//! # How it works
//!
//! The first [`MullionOverlay`] to mount lazily creates a single overlay root
//! (`<div id="mullion-overlay-root">`) and appends it to `document.body`, after
//! the mullion root. It is `position:fixed; inset:0` with
//! `z-index: var(--ml-overlay-z, 10000)`, so it is a stacking context sitting
//! above the whole chrome band. It is `pointer-events:none`, so an idle overlay
//! layer never eats clicks.
//!
//! The default 10000 is deliberately above the `z-index: 9999` idiom, so an
//! overlay still wins against app content that has not yet been ported off
//! hand-picked z-indexes. A host with its own high-z chrome (a global toast
//! rail, say) can move the whole layer by defining `--ml-overlay-z`.
//!
//! Each `MullionOverlay` portals into that root and wraps its children in a
//! `position:fixed; inset:0` box carrying an [`OverlayLevel`] z-index. Because
//! that box is itself a stacking context, whatever z-indexes the overlay's own
//! content uses stay contained inside it — an app's internal `z-index: 9999`
//! keeps working relative to its siblings and can never leak back over chrome.
//!
//! Reactive context is preserved across the portal: Leptos's `<Portal>` mounts
//! children under the current reactive owner, so `use_context`, signals, and
//! effects behave exactly as they do inline.
//!
//! ```ignore
//! view! {
//!     <Show when=move || open.get()>
//!         <MullionOverlay backdrop=true on_backdrop_click=Callback::new(move |_| open.set(false))>
//!             <div style="position:absolute;inset:0;display:grid;place-items:center">
//!                 <MyDialog />
//!             </div>
//!         </MullionOverlay>
//!     </Show>
//! }
//! ```

use leptos::portal::Portal;
use leptos::prelude::*;

/// Id of the singleton overlay root appended to `document.body`.
pub const OVERLAY_ROOT_ID: &str = "mullion-overlay-root";

/// Stacking tier within mullion's overlay layer.
///
/// Tiers order overlays against each other. Every tier is above all pane
/// chrome, so picking one is only a question of what should win when two
/// overlays are open at once.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OverlayLevel {
    /// App-owned modals, dialogs, command palettes. The default.
    #[default]
    Modal,
    /// Transient app feedback that should sit above an open modal (toasts).
    Toast,
    /// Drag/drop feedback. Reserved for mullion; above everything else.
    Drag,
}

impl OverlayLevel {
    /// z-index inside the overlay root's stacking context.
    fn z(self) -> u16 {
        match self {
            OverlayLevel::Modal => 10,
            OverlayLevel::Toast => 20,
            OverlayLevel::Drag => 30,
        }
    }
}

/// Returns the singleton overlay root, creating and appending it on first use.
fn overlay_root() -> Option<web_sys::Element> {
    let doc = document();
    if let Some(existing) = doc.get_element_by_id(OVERLAY_ROOT_ID) {
        return Some(existing);
    }
    let el = doc.create_element("div").ok()?;
    el.set_id(OVERLAY_ROOT_ID);
    let _ = el.set_attribute(
        "style",
        // `pointer-events:none` so the always-present layer is transparent to
        // hit testing; each overlay opts its own box back in.
        "position:fixed;inset:0;z-index:var(--ml-overlay-z, 10000);pointer-events:none",
    );
    doc.body()?.append_child(&el).ok()?;
    Some(el)
}

/// Renders `children` in mullion's overlay layer, above all pane chrome.
///
/// Use this for anything that must escape its pane — full-screen modals,
/// command palettes, dialogs. The overlay is only rendered while this component
/// is mounted, so drive visibility with `<Show>` (or a conditional) around it
/// rather than a `display:none` inside it.
///
/// Overlays do not pick z-indexes; they pick a [`OverlayLevel`]. See the
/// [module docs](self) for the layering rules.
#[component]
pub fn MullionOverlay(
    /// Stacking tier against other overlays. Defaults to [`OverlayLevel::Modal`].
    #[prop(optional)]
    level: OverlayLevel,
    /// Render a dimming backdrop behind `children`. Defaults to `false`.
    #[prop(optional)]
    backdrop: bool,
    /// Backdrop color. Defaults to `var(--ml-scrim, rgba(0,0,0,0.5))`, so an app
    /// can theme every mullion backdrop by defining `--ml-scrim`.
    #[prop(optional, into)]
    backdrop_color: Option<String>,
    /// Center `children` in the viewport. Convenience for the common
    /// "dim the app, float a panel" shape; leave it off to lay the overlay out
    /// yourself.
    #[prop(optional)]
    center: bool,
    /// Called when a click lands inside the overlay but not on anything
    /// `children` rendered — the usual "click outside to dismiss".
    ///
    /// Detection is by event target, so it only fires for clicks that reach the
    /// overlay's own box. If `children` themselves fill the overlay (e.g. an
    /// `inset:0` wrapper of your own), every click lands on *your* element and
    /// this never fires — use `center`, or handle dismissal yourself.
    #[prop(optional)]
    on_click_outside: Option<Callback<()>>,
    /// Let pointer events pass through to the app behind the overlay. Off by
    /// default (modal semantics: the overlay blocks the app underneath).
    /// Content inside must opt back in with `pointer-events:auto`.
    #[prop(optional)]
    click_through: bool,
    children: ChildrenFn,
) -> impl IntoView {
    let Some(root) = overlay_root() else {
        return ().into_any();
    };

    let pointer = if click_through { "none" } else { "auto" };
    let wrapper_style = format!(
        // The z-index makes this box a stacking context, containing whatever
        // the overlay's own content does with z-index.
        "position:fixed;inset:0;z-index:{};pointer-events:{pointer}",
        level.z()
    );

    let backdrop_style = format!(
        "position:absolute;inset:0;z-index:0;background:{}",
        backdrop_color.unwrap_or_else(|| "var(--ml-scrim, rgba(0,0,0,0.5))".into())
    );

    // The content layer covers the backdrop, so "click outside" is detected
    // here rather than on the backdrop: a click whose target is this element
    // itself landed on empty overlay, not on anything the caller rendered.
    let content_style = if center {
        "position:relative;z-index:1;width:100%;height:100%;display:grid;place-items:center"
    } else {
        "position:relative;z-index:1;width:100%;height:100%"
    };

    view! {
        <Portal mount=root>
            <div style=wrapper_style.clone()>
                {backdrop.then(|| view! { <div style=backdrop_style.clone()></div> })}
                <div
                    style=content_style
                    on:click=move |ev| {
                        let Some(cb) = on_click_outside else { return };
                        if ev.target().is_some_and(|t| {
                            ev.current_target().is_some_and(|c| t == c)
                        }) {
                            cb.run(());
                        }
                    }
                >
                    {children()}
                </div>
            </div>
        </Portal>
    }
    .into_any()
}
