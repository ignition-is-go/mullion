use leptos::prelude::*;

use css_styled::{IntoCss, IntoThemeCss};

use crate::activity::Category;
use crate::context::MullionContext;
use crate::events::PaneEvent;
use crate::theme::MullionTheme;
use crate::tree::{PaneData, PaneNode};

use super::activity_bar::ActivityBarStyle;
use super::drop_overlay::DropOverlayStyle;
use super::pane_view::{PaneStyle, PaneView};
use super::split_handle::SplitHandleStyle;

/// Style for the mullion root container, powered by css-styled.
#[derive(css_styled::StyledComponent, Clone, Debug)]
#[component(scope = "mullion-root")]
#[component(theme = MullionTheme)]
#[component(base_css)]
pub struct MullionStyle {
    #[prop(css = "background", default = theme.bg)]
    pub background: String,
}

impl css_styled::StyledComponentBase for MullionStyle {
    fn base_css() -> &'static str {
        css_styled::css!(MullionStyle, {
            SCOPE {
                width: 100%;
                height: 100%;
            }
        })
    }
}

/// Context-only provider for the mullion pane system.
///
/// Sets up `MullionContext` and default themes, then renders its children.
/// Use this when you want full control over layout (e.g., placing a
/// `WorkspaceSwitcher` alongside the pane tree).
///
/// Children can access `MullionContext<D>` via `use_context`.
#[component]
pub fn MullionProvider<D: PaneData + Send + Sync>(
    /// The initial pane tree layout.
    initial_tree: PaneNode<D>,
    /// Categories with their activities.
    categories: Vec<Category<D>>,
    /// Called for every pane event (split, close, move, resize, etc.).
    on_event: impl Fn(PaneEvent<D>) + Send + Sync + 'static,
    /// Optional upstream signal to update the tree live from server queries.
    #[prop(optional)]
    upstream: Option<ReadSignal<Option<PaneNode<D>>>>,
    /// Optional app icon shown at the top of every activity bar.
    #[prop(optional)]
    app_icon: Option<crate::activity::ActivityIcon>,
    children: Children,
) -> impl IntoView {
    let theme = use_context::<MullionTheme>().unwrap_or_default();
    let mullion_style = use_context::<MullionStyle>().unwrap_or_default();
    let activity_bar_style = use_context::<ActivityBarStyle>().unwrap_or_default();
    let split_handle_style = use_context::<SplitHandleStyle>().unwrap_or_default();
    let pane_style = use_context::<PaneStyle>().unwrap_or_default();
    let drop_overlay_style = use_context::<DropOverlayStyle>().unwrap_or_default();

    let all_css = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        theme.to_theme_css(),
        split_handle_style.to_css(),
        pane_style.to_css(),
        mullion_style.to_css(),
        activity_bar_style.to_css(),
        drop_overlay_style.to_css(),
    );

    let ctx = MullionContext::new(
        initial_tree,
        categories,
        on_event,
        theme,
        mullion_style,
        activity_bar_style,
        split_handle_style,
        pane_style,
        drop_overlay_style,
        app_icon,
    );

    if let Some(upstream_sig) = upstream {
        let ctx_clone = ctx.clone();
        Effect::new(move |_| {
            if let Some(new_tree) = upstream_sig.get() {
                ctx_clone.set_tree(new_tree);
            }
        });
    }

    provide_context(ctx);

    view! {
        <style>{all_css}</style>
        {children()}
    }
}

/// All-in-one component: provides context and renders the pane tree.
#[component]
pub fn MullionRoot<D: PaneData + Send + Sync>(
    /// The initial pane tree layout.
    initial_tree: PaneNode<D>,
    /// Categories with their activities.
    categories: Vec<Category<D>>,
    /// Called for every pane event.
    on_event: impl Fn(PaneEvent<D>) + Send + Sync + 'static,
    /// Optional upstream signal.
    #[prop(optional)]
    upstream: Option<ReadSignal<Option<PaneNode<D>>>>,
    /// Optional app icon shown at the top of every activity bar.
    #[prop(optional)]
    app_icon: Option<crate::activity::ActivityIcon>,
) -> impl IntoView {
    let theme = use_context::<MullionTheme>().unwrap_or_default();
    let mullion_style = use_context::<MullionStyle>().unwrap_or_default();
    let activity_bar_style = use_context::<ActivityBarStyle>().unwrap_or_default();
    let split_handle_style = use_context::<SplitHandleStyle>().unwrap_or_default();
    let pane_style = use_context::<PaneStyle>().unwrap_or_default();
    let drop_overlay_style = use_context::<DropOverlayStyle>().unwrap_or_default();

    let all_css = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        theme.to_theme_css(),
        split_handle_style.to_css(),
        pane_style.to_css(),
        mullion_style.to_css(),
        activity_bar_style.to_css(),
        drop_overlay_style.to_css(),
    );

    let ctx = MullionContext::new(
        initial_tree,
        categories,
        on_event,
        theme,
        mullion_style,
        activity_bar_style,
        split_handle_style,
        pane_style,
        drop_overlay_style,
        app_icon,
    );

    if let Some(upstream_sig) = upstream {
        let ctx_clone = ctx.clone();
        Effect::new(move |_| {
            if let Some(new_tree) = upstream_sig.get() {
                ctx_clone.set_tree(new_tree);
            }
        });
    }

    provide_context(ctx.clone());

    let tree = ctx.tree;

    view! {
        <style>{all_css}</style>
        <div class=MullionStyle::SCOPE>
            {move || {
                let current_tree = tree.get();
                view! { <PaneView node=current_tree ctx=ctx.clone() /> }
            }}
        </div>
    }
}

/// Renders just the pane tree from a `MullionContext`.
#[component]
pub fn MullionPaneTree<D: PaneData + Send + Sync>(
    ctx: MullionContext<D>,
) -> impl IntoView {
    let all_css = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        ctx.theme.to_theme_css(),
        ctx.split_handle_style.to_css(),
        ctx.pane_style.to_css(),
        ctx.mullion_style.to_css(),
        ctx.activity_bar_style.to_css(),
        ctx.drop_overlay_style.to_css(),
    );
    let tree = ctx.tree;

    view! {
        <style>{all_css}</style>
        <div class=MullionStyle::SCOPE>
            {move || {
                let current_tree = tree.get();
                view! { <PaneView node=current_tree ctx=ctx.clone() /> }
            }}
        </div>
    }
}
