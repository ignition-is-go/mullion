use leptos::prelude::*;

use css_styled::{IntoCss, IntoThemeCss};

use crate::activity::Category;
use crate::context::{MullionContext, PaneAccessory, PaneBorderColor};
use crate::events::PaneEvent;
use crate::theme::MullionTheme;
use crate::tree::{PaneData, PaneNode};

use super::activity_bar::{ActivityBarBehavior, ActivityBarStyle};
use super::drop_overlay::DropOverlayStyle;
use super::pane_header::HeaderStyle;
use super::pane_view::{PaneStyle, PaneView};
use super::split_handle::SplitHandleStyle;
use super::workspace_switcher::WorkspaceSwitcherStyle;

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
    /// Optional per-pane chrome rendered in each activity bar's bottom action
    /// area (e.g. a session-color indicator/switcher). Closes over host state.
    #[prop(optional)]
    pane_accessory: Option<PaneAccessory>,
    /// Optional per-pane bottom-border color (e.g. the pane's session color).
    #[prop(optional)]
    pane_border_color: Option<PaneBorderColor>,
    /// Activities registered outside any category — rendered as top-level icons
    /// in every activity bar, above the categories. Default: none.
    #[prop(optional)]
    floating_activities: Vec<crate::activity::ActivityDef<D>>,
    /// Whether panes render their header band (the active activity's title).
    /// Default: `true`.
    #[prop(default = true)]
    show_pane_header: bool,
    /// Optional predicate: panes for which it returns `true` hide their activity
    /// bar (getting hover controls instead). Default: every pane keeps its bar.
    #[prop(optional)]
    hide_activity_bar: Option<crate::context::PaneHideActivityBar<D>>,
    /// Optional predicate: panes for which it returns `true` auto-hide their
    /// activity bar off the left edge (revealed on edge-hover), while keeping the
    /// bar. Default: every pane's bar is pinned/visible.
    #[prop(optional)]
    auto_hide_activity_bar: Option<crate::context::PaneAutoHideActivityBar<D>>,
    children: Children,
) -> impl IntoView {
    let theme = use_context::<MullionTheme>().unwrap_or_default();
    let mullion_style = use_context::<MullionStyle>().unwrap_or_default();
    let activity_bar_style = use_context::<ActivityBarStyle>().unwrap_or_default();
    let split_handle_style = use_context::<SplitHandleStyle>().unwrap_or_default();
    let pane_style = use_context::<PaneStyle>().unwrap_or_default();
    let drop_overlay_style = use_context::<DropOverlayStyle>().unwrap_or_default();
    let header_style = use_context::<HeaderStyle>().unwrap_or_default();
    let ws_style = use_context::<WorkspaceSwitcherStyle>().unwrap_or_default();
    let activity_bar_behavior = use_context::<ActivityBarBehavior>().unwrap_or_default();

    let all_css = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        theme.to_theme_css(),
        split_handle_style.to_css(),
        pane_style.to_css(),
        mullion_style.to_css(),
        activity_bar_style.to_css(),
        drop_overlay_style.to_css(),
        header_style.to_css(),
        ws_style.to_css(),
    );

    let ctx = MullionContext::new(
        initial_tree,
        categories,
        floating_activities,
        on_event,
        theme,
        mullion_style,
        activity_bar_style,
        split_handle_style,
        pane_style,
        drop_overlay_style,
        header_style,
        activity_bar_behavior,
        app_icon,
        pane_accessory,
        pane_border_color,
        show_pane_header,
        hide_activity_bar,
        auto_hide_activity_bar,
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
    /// Optional per-pane chrome rendered in each activity bar's bottom action
    /// area (e.g. a session-color indicator/switcher). Closes over host state.
    #[prop(optional)]
    pane_accessory: Option<PaneAccessory>,
    /// Optional per-pane bottom-border color (e.g. the pane's session color).
    #[prop(optional)]
    pane_border_color: Option<PaneBorderColor>,
    /// Activities registered outside any category — rendered as top-level icons
    /// in every activity bar, above the categories. Default: none.
    #[prop(optional)]
    floating_activities: Vec<crate::activity::ActivityDef<D>>,
    /// Whether panes render their header band (the active activity's title).
    /// Default: `true`.
    #[prop(default = true)]
    show_pane_header: bool,
    /// Optional predicate: panes for which it returns `true` hide their activity
    /// bar (getting hover controls instead). Default: every pane keeps its bar.
    #[prop(optional)]
    hide_activity_bar: Option<crate::context::PaneHideActivityBar<D>>,
    /// Optional predicate: panes for which it returns `true` auto-hide their
    /// activity bar off the left edge (revealed on edge-hover), while keeping the
    /// bar. Default: every pane's bar is pinned/visible.
    #[prop(optional)]
    auto_hide_activity_bar: Option<crate::context::PaneAutoHideActivityBar<D>>,
) -> impl IntoView {
    let theme = use_context::<MullionTheme>().unwrap_or_default();
    let mullion_style = use_context::<MullionStyle>().unwrap_or_default();
    let activity_bar_style = use_context::<ActivityBarStyle>().unwrap_or_default();
    let split_handle_style = use_context::<SplitHandleStyle>().unwrap_or_default();
    let pane_style = use_context::<PaneStyle>().unwrap_or_default();
    let drop_overlay_style = use_context::<DropOverlayStyle>().unwrap_or_default();
    let header_style = use_context::<HeaderStyle>().unwrap_or_default();
    let ws_style = use_context::<WorkspaceSwitcherStyle>().unwrap_or_default();
    let activity_bar_behavior = use_context::<ActivityBarBehavior>().unwrap_or_default();

    let all_css = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        theme.to_theme_css(),
        split_handle_style.to_css(),
        pane_style.to_css(),
        mullion_style.to_css(),
        activity_bar_style.to_css(),
        drop_overlay_style.to_css(),
        header_style.to_css(),
        ws_style.to_css(),
    );

    let ctx = MullionContext::new(
        initial_tree,
        categories,
        floating_activities,
        on_event,
        theme,
        mullion_style,
        activity_bar_style,
        split_handle_style,
        pane_style,
        drop_overlay_style,
        header_style,
        activity_bar_behavior,
        app_icon,
        pane_accessory,
        pane_border_color,
        show_pane_header,
        hide_activity_bar,
        auto_hide_activity_bar,
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

    view! {
        <style>{all_css}</style>
        <div class=MullionStyle::SCOPE>
            <PaneView ctx=ctx />
        </div>
    }
}

/// Renders just the pane tree from a `MullionContext`.
#[component]
pub fn MullionPaneTree<D: PaneData + Send + Sync>(ctx: MullionContext<D>) -> impl IntoView {
    let all_css = format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        ctx.theme.to_theme_css(),
        ctx.split_handle_style.to_css(),
        ctx.pane_style.to_css(),
        ctx.mullion_style.to_css(),
        ctx.activity_bar_style.to_css(),
        ctx.drop_overlay_style.to_css(),
        ctx.header_style.to_css(),
    );

    view! {
        <style>{all_css}</style>
        <div class=MullionStyle::SCOPE>
            <PaneView ctx=ctx />
        </div>
    }
}
