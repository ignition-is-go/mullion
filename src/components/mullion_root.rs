use leptos::prelude::*;

use super::split_handle::{CLASS_SPLIT_BAR, CLASS_SPLIT_TARGET};

fn split_handle_hover_css(theme: &SplitHandleTheme) -> String {
    format!(
        ".{target}:hover .{bar} {{ background: {color} !important; }}",
        target = CLASS_SPLIT_TARGET,
        bar = CLASS_SPLIT_BAR,
        color = theme.hover_color,
    )
}

use crate::activity::Category;
use crate::context::MullionContext;
use crate::events::PaneEvent;
use crate::theme::{ActivityBarTheme, DropOverlayTheme, MullionTheme, PaneTheme, SplitHandleTheme};
use crate::tree::{PaneData, PaneNode};

use super::pane_view::PaneView;

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
    let mullion_theme = use_context::<MullionTheme>().unwrap_or_default();
    let activity_bar_theme = use_context::<ActivityBarTheme>().unwrap_or_default();
    let split_handle_theme = use_context::<SplitHandleTheme>().unwrap_or_default();
    let pane_theme = use_context::<PaneTheme>().unwrap_or_default();
    let drop_overlay_theme = use_context::<DropOverlayTheme>().unwrap_or_default();

    let css = split_handle_hover_css(&split_handle_theme);

    let ctx = MullionContext::new(
        initial_tree,
        categories,
        on_event,
        mullion_theme,
        activity_bar_theme,
        split_handle_theme,
        pane_theme,
        drop_overlay_theme,
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
        <style>{css}</style>
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
    let mullion_theme = use_context::<MullionTheme>().unwrap_or_default();
    let activity_bar_theme = use_context::<ActivityBarTheme>().unwrap_or_default();
    let split_handle_theme = use_context::<SplitHandleTheme>().unwrap_or_default();
    let pane_theme = use_context::<PaneTheme>().unwrap_or_default();
    let drop_overlay_theme = use_context::<DropOverlayTheme>().unwrap_or_default();

    let css = split_handle_hover_css(&split_handle_theme);
    let root_style = format!("width:100%;height:100%;background:{}", mullion_theme.background);

    let ctx = MullionContext::new(
        initial_tree,
        categories,
        on_event,
        mullion_theme,
        activity_bar_theme,
        split_handle_theme,
        pane_theme,
        drop_overlay_theme,
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
        <style>{css}</style>
        <div style={root_style}>
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
    let root_style = format!("width:100%;height:100%;background:{}", ctx.mullion_theme.background);
    let css = split_handle_hover_css(&ctx.split_handle_theme);
    let tree = ctx.tree;

    view! {
        <style>{css}</style>
        <div style={root_style}>
            {move || {
                let current_tree = tree.get();
                view! { <PaneView node=current_tree ctx=ctx.clone() /> }
            }}
        </div>
    }
}
