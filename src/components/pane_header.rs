use leptos::prelude::*;

use crate::context::MullionContext;
use crate::theme::MullionTheme;
use crate::tree::{ActivityId, PaneData, PaneId};

/// Style for the per-pane header band, powered by css-styled.
///
/// The header band sits above each pane's activity body and shows the active
/// activity's name plus any custom content the activity defines (see
/// [`crate::ActivityDef::header`]). All customizable values are CSS custom
/// properties so consumers can theme it to their own tokens; structural
/// layout comes from base CSS.
#[derive(css_styled::StyledComponent, Clone, Debug)]
#[component(scope = "mullion-header")]
#[component(theme = MullionTheme)]
#[component(class(title = "mullion-header-title", content = "mullion-header-content"))]
#[component(base_css)]
pub struct HeaderStyle {
    #[prop(var = "--mh-height", default = "28px")]
    pub height: String,
    #[prop(var = "--mh-background", default = theme.surface)]
    pub background: String,
    #[prop(var = "--mh-color", default = theme.text)]
    pub color: String,
    #[prop(var = "--mh-border", default = "1px solid var(--ml-border)")]
    pub border: String,
    #[prop(var = "--mh-padding", default = "0 8px")]
    pub padding: String,
    #[prop(var = "--mh-font-size", default = "11px")]
    pub font_size: String,
    #[prop(var = "--mh-gap", default = "8px")]
    pub gap: String,
    #[prop(var = "--mh-title-color", default = theme.text)]
    pub title_color: String,
    #[prop(var = "--mh-title-weight", default = "600")]
    pub title_weight: String,
}

impl css_styled::StyledComponentBase for HeaderStyle {
    fn base_css() -> &'static str {
        css_styled::css!(HeaderStyle, {
            SCOPE {
                flex-shrink: 0;
                display: flex;
                align-items: center;
                gap: var(--mh-gap);
                height: var(--mh-height);
                min-height: var(--mh-height);
                padding: var(--mh-padding);
                background: var(--mh-background);
                color: var(--mh-color);
                border-bottom: var(--mh-border);
                font-size: var(--mh-font-size);
                overflow: hidden;
                white-space: nowrap;
            }
            TITLE {
                flex-shrink: 0;
                font-weight: var(--mh-title-weight);
                color: var(--mh-title-color);
                overflow: hidden;
                text-overflow: ellipsis;
            }
            CONTENT {
                display: flex;
                align-items: center;
                gap: var(--mh-gap);
                min-width: 0;
                overflow: hidden;
                text-overflow: ellipsis;
            }
        })
    }
}

/// Renders the header band for a pane.
///
/// Shows the resolved (active) activity's name, plus any custom content that
/// activity defines via [`crate::ActivityDef::header`]. Subscribes reactively
/// to `resolved` — the band re-renders when the active activity changes. The
/// custom content receives the same `Signal<D>` as the activity body, so it
/// updates independently when this pane's data changes.
///
/// Renders nothing when no activity is resolved (e.g. an empty pane), so panes
/// with no available activities show no band.
#[component]
pub fn PaneHeader<D: PaneData + Send + Sync>(
    pane_id: PaneId,
    resolved: Signal<Option<ActivityId>>,
    data: Signal<D>,
    ctx: MullionContext<D>,
) -> impl IntoView {
    view! {
        {move || {
            let ctx = ctx.clone();
            let pane_id = pane_id.clone();
            let id = resolved.get();
            id.and_then(move |id| {
                ctx.activities.with_value(|acts| {
                    acts.iter()
                        .find(|a| a.def.id == id)
                        .map(|a| (a.def.name.clone(), a.def.header))
                })
            })
            .map(|(name, header_fn)| {
                let custom = header_fn.map(|h| h(pane_id.clone(), data));
                view! {
                    <div class=HeaderStyle::SCOPE>
                        <span class=HeaderStyle::TITLE>{name}</span>
                        {custom.map(|c| view! { <div class=HeaderStyle::CONTENT>{c}</div> })}
                    </div>
                }
            })
        }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use css_styled::IntoCss;

    #[test]
    fn base_css_lays_out_header_band() {
        let css = HeaderStyle::default().to_css();
        // The band must be a flex row that doesn't shrink away in the
        // surrounding flex column.
        assert!(
            css.contains("flex-shrink: 0"),
            "expected flex-shrink:0, got: {css}"
        );
        assert!(
            css.contains("--mh-height"),
            "expected height var, got: {css}"
        );
    }

    #[test]
    fn class_names_are_scoped() {
        assert_eq!(HeaderStyle::TITLE, "mullion-header-title");
        assert_eq!(HeaderStyle::CONTENT, "mullion-header-content");
    }
}
