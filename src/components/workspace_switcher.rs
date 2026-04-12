use leptos::prelude::*;

use crate::context::MullionContext;
use crate::tree::PaneData;
use crate::workspace::WorkspaceManager;

/// Theme for the workspace switcher component.
#[derive(Clone, Debug)]
pub struct WorkspaceSwitcherTheme {
    pub button_background: String,
    pub button_color: String,
    pub button_active_background: String,
    pub button_active_color: String,
    pub gap: String,
    pub font_size: String,
    pub padding: String,
    pub border_radius: String,
}

impl Default for WorkspaceSwitcherTheme {
    fn default() -> Self {
        WorkspaceSwitcherTheme {
            button_background: "#3c3c3c".into(),
            button_color: "#ccc".into(),
            button_active_background: "#007acc".into(),
            button_active_color: "#fff".into(),
            gap: "4px".into(),
            font_size: "12px".into(),
            padding: "4px 12px".into(),
            border_radius: "3px".into(),
        }
    }
}

/// A batteries-included workspace switcher component.
#[component]
pub fn WorkspaceSwitcher<D: PaneData + Send + Sync>(
    manager: WorkspaceManager<D>,
    ctx: MullionContext<D>,
) -> impl IntoView {
    let theme = use_context::<WorkspaceSwitcherTheme>().unwrap_or_default();

    let container_style = format!("display:flex;gap:{}", theme.gap);

    let workspaces = manager.workspaces_signal();
    let active = manager.active_signal();

    view! {
        <div style={container_style}>
            {move || {
                let ws_list = workspaces.get();
                let current = active.get();

                ws_list.into_iter().map(|ws| {
                    let ws_id = ws.id.clone();
                    let is_active = ws_id == current;
                    let bg = if is_active { theme.button_active_background.clone() } else { theme.button_background.clone() };
                    let color = if is_active { theme.button_active_color.clone() } else { theme.button_color.clone() };
                    let style = format!(
                        "background:{};color:{};border:none;padding:{};border-radius:{};font-size:{};cursor:pointer",
                        bg, color, theme.padding, theme.border_radius, theme.font_size
                    );

                    let manager = manager.clone();
                    let ctx = ctx.clone();
                    let ws_id_click = ws_id.clone();

                    view! {
                        <button
                            style={style}
                            on:click=move |_| {
                                if let Some(tree) = manager.switch_to(&ws_id_click) {
                                    ctx.set_tree(tree);
                                }
                            }
                        >
                            {ws.name}
                        </button>
                    }
                }).collect::<Vec<_>>()
            }}
        </div>
    }
}
