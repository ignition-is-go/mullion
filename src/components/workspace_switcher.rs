use leptos::prelude::*;

use crate::context::MullionContext;
use crate::theme::MullionTheme;
use crate::tree::PaneData;
use crate::workspace::WorkspaceManager;

/// Style for the workspace switcher component.
#[derive(css_styled::StyledComponent, Clone, Debug)]
#[component(scope = "mullion-ws")]
#[component(theme = MullionTheme)]
#[component(class(btn = "mullion-ws-btn"))]
#[component(modifier(active))]
#[component(base_css)]
pub struct WorkspaceSwitcherStyle {
    #[prop(var = "--ws-btn-bg", default = theme.accent)]
    pub button_background: String,
    #[prop(var = "--ws-btn-color", default = theme.text_muted)]
    pub button_color: String,
    #[prop(var = "--ws-btn-active-bg", default = theme.highlight)]
    pub button_active_background: String,
    #[prop(var = "--ws-btn-active-color", default = theme.text)]
    pub button_active_color: String,
    #[prop(var = "--ws-gap", default = "4px")]
    pub gap: String,
    #[prop(var = "--ws-font-size", default = "12px")]
    pub font_size: String,
    #[prop(var = "--ws-padding", default = "4px 12px")]
    pub padding: String,
    #[prop(var = "--ws-border-radius", default = "3px")]
    pub border_radius: String,
}

impl css_styled::StyledComponentBase for WorkspaceSwitcherStyle {
    fn base_css() -> &'static str {
        css_styled::css!(WorkspaceSwitcherStyle, {
            SCOPE {
                display: flex;
                gap: var(--ws-gap);
            }
            BTN {
                border: none;
                padding: var(--ws-padding);
                border-radius: var(--ws-border-radius);
                font-size: var(--ws-font-size);
                cursor: pointer;
                background: var(--ws-btn-bg);
                color: var(--ws-btn-color);
            }
            BTN.ACTIVE {
                background: var(--ws-btn-active-bg);
                color: var(--ws-btn-active-color);
            }
        })
    }
}

/// A batteries-included workspace switcher component.
#[component]
pub fn WorkspaceSwitcher<D: PaneData + Send + Sync>(
    manager: WorkspaceManager<D>,
    ctx: MullionContext<D>,
) -> impl IntoView {
    let workspaces = manager.workspaces_signal();
    let active = manager.active_signal();

    view! {
        <div class=WorkspaceSwitcherStyle::SCOPE>
            {move || {
                let ws_list = workspaces.get();
                let current = active.get();

                ws_list.into_iter().map(|ws| {
                    let ws_id = ws.id.clone();
                    let is_active = ws_id == current;
                    let class = if is_active {
                        WorkspaceSwitcherStyle::class(&[WorkspaceSwitcherModifier::Active])
                    } else {
                        WorkspaceSwitcherStyle::BTN.to_string()
                    };

                    let manager = manager.clone();
                    let ctx = ctx.clone();
                    let ws_id_click = ws_id.clone();

                    view! {
                        <button
                            class=class
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
