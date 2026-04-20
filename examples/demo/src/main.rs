use css_styled::{StyledComponent, StyledComponentBase, css, IntoCss};
use leptos::prelude::*;
use md_icons::outlined;
use mullion::*;
use serde::{Deserialize, Serialize};

// ── Demo styles ──────────────────────────────────────────────────────────────

#[derive(StyledComponent, Clone, Debug)]
#[component(scope = "demo-layout")]
#[component(theme = MullionTheme)]
#[component(class(content = "demo-layout-content", footer = "demo-layout-footer"))]
#[component(base_css)]
struct DemoLayoutStyle {
    #[prop(var = "--demo-footer-bg", default = theme.bg)]
    pub footer_bg: String,
    #[prop(var = "--demo-footer-border", default = "1px solid var(--ml-border)")]
    pub footer_border: String,
}

impl StyledComponentBase for DemoLayoutStyle {
    fn base_css() -> &'static str {
        css!(DemoLayoutStyle, {
            SCOPE {
                display: flex;
                flex-direction: column;
                width: 100vw;
                height: 100vh;
            }
            CONTENT {
                flex: 1;
                min-height: 0;
                overflow: hidden;
            }
            FOOTER {
                display: flex;
                gap: 1px;
                background: var(--demo-footer-bg);
                padding: 2px 4px;
                border-top: var(--demo-footer-border);
            }
        })
    }
}

#[derive(StyledComponent, Clone, Debug)]
#[component(scope = "demo-tab")]
#[component(theme = MullionTheme)]
#[component(modifier(active))]
#[component(base_css)]
struct FooterTabStyle {
    #[prop(var = "--tab-bg", default = "transparent")]
    pub bg: String,
    #[prop(var = "--tab-color", default = theme.text_muted)]
    pub color: String,
    #[prop(var = "--tab-active-bg", default = theme.accent)]
    pub active_bg: String,
    #[prop(var = "--tab-active-color", default = theme.text)]
    pub active_color: String,
}

impl StyledComponentBase for FooterTabStyle {
    fn base_css() -> &'static str {
        css!(FooterTabStyle, {
            SCOPE {
                background: var(--tab-bg);
                color: var(--tab-color);
                border: none;
                padding: 2px 8px;
                font-size: 11px;
                cursor: pointer;
                border-radius: 2px;
                font-family: monospace;
            }
            SCOPE.ACTIVE {
                background: var(--tab-active-bg);
                color: var(--tab-active-color);
            }
        })
    }
}

#[derive(StyledComponent, Clone, Debug)]
#[component(scope = "demo-input")]
#[component(theme = MullionTheme)]
#[component(base_css)]
struct InputStyle {
    #[prop(var = "--input-bg", default = theme.accent)]
    pub bg: String,
    #[prop(var = "--input-border", default = "1px solid var(--ml-highlight)")]
    pub border: String,
    #[prop(var = "--input-color", default = theme.text)]
    pub color: String,
}

impl StyledComponentBase for InputStyle {
    fn base_css() -> &'static str {
        css!(InputStyle, {
            SCOPE {
                width: 100%;
                padding: 6px 8px;
                background: var(--input-bg);
                border: var(--input-border);
                color: var(--input-color);
                border-radius: 3px;
                margin-top: 8px;
            }
        })
    }
}

// ── Data ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct DemoData {
    label: String,
    show_files: bool,
    show_search: bool,
    show_settings: bool,
}

impl Default for DemoData {
    fn default() -> Self {
        DemoData {
            label: "Pane".into(),
            show_files: true,
            show_search: true,
            show_settings: true,
        }
    }
}

fn categories() -> Vec<Category<DemoData>> {
    vec![
        Category {
            id: CategoryId::new("0"), name: "Explorer".into(), order: 0,
            icon: ActivityIcon::Svg(outlined::ICON_FOLDER.into()),
            color: "#75beff".into(),
            activities: vec![
                ActivityDef {
                    id: ActivityId::new("1"), name: "Files".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_DESCRIPTION.into()),
                    filter: |d| d.show_files, render: |_pid, data| view! { <FilesActivity data=data /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId::new("2"), name: "Open Editors".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_ARTICLE.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Open Editors" /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId::new("3"), name: "Timeline".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_TIMELINE.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Timeline" /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId::new("4"), name: "Outline".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_LIST.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Outline" /> }.into_any(),
                },
            ],
        },
        Category {
            id: CategoryId::new("1"), name: "Edit".into(), order: 1,
            icon: ActivityIcon::Svg(outlined::ICON_EDIT_NOTE.into()),
            color: "#e8ab53".into(),
            activities: vec![
                ActivityDef {
                    id: ActivityId::new("5"), name: "Search".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_SEARCH.into()),
                    filter: |d| d.show_search, render: |_pid, data| view! { <SearchActivity data=data /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId::new("6"), name: "Replace".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_FIND_REPLACE.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Replace" /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId::new("7"), name: "Bookmarks".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_BOOKMARKS.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Bookmarks" /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId::new("8"), name: "Snippets".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_CODE.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Snippets" /> }.into_any(),
                },
            ],
        },
        Category {
            id: CategoryId::new("2"), name: "Preferences".into(), order: 2,
            icon: ActivityIcon::Svg(outlined::ICON_SETTINGS.into()),
            color: "#c586c0".into(),
            activities: vec![
                ActivityDef {
                    id: ActivityId::new("9"), name: "Settings".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_SETTINGS.into()),
                    filter: |d| d.show_settings, render: |_pid, _data| view! { <SettingsActivity /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId::new("10"), name: "Themes".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_PALETTE.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Themes" /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId::new("11"), name: "Keybindings".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_KEYBOARD.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Keybindings" /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId::new("12"), name: "Extensions".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_EXTENSION.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Extensions" /> }.into_any(),
                },
            ],
        },
    ]
}

// ── Activity content views ───────────────────────────────────────────────────

#[component]
fn FilesActivity(data: Signal<DemoData>) -> impl IntoView {
    let files = vec![
        "src/main.rs", "src/lib.rs", "src/components/mod.rs",
        "src/components/header.rs", "src/components/sidebar.rs",
        "Cargo.toml", "README.md",
    ];
    view! {
        <div class="activity-content">
            <h2>{move || data.get().label} " - Files"</h2>
            <div>
                {files.into_iter().map(|f| view! { <div class="file-item">{f}</div> }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}

#[component]
fn SearchActivity(data: Signal<DemoData>) -> impl IntoView {
    view! {
        <div class="activity-content">
            <h2>{move || data.get().label} " - Search"</h2>
            <p>"Type to search across files..."</p>
            <input type="text" placeholder="Search..." class=InputStyle::SCOPE />
        </div>
    }
}

#[component]
fn PlaceholderActivity(name: &'static str) -> impl IntoView {
    view! {
        <div class="activity-content">
            <h2>{name}</h2>
            <p>"This activity is a placeholder."</p>
        </div>
    }
}

#[component]
fn SettingsActivity() -> impl IntoView {
    view! {
        <div class="activity-content">
            <h2>"Settings"</h2>
            <p>"Editor preferences and configuration."</p>
        </div>
    }
}

// ── Workspaces ───────────────────────────────────────────────────────────────

fn default_workspace() -> PaneNode<DemoData> {
    PaneNode::Split {
        direction: SplitDirection::Horizontal,
        ratio: 0.5,
        first: Box::new(PaneNode::leaf_with_activity(PaneId::new("1"), ActivityId::new("1"),
            DemoData { label: "Left".into(), ..Default::default() })),
        second: Box::new(PaneNode::leaf_with_activity(PaneId::new("2"), ActivityId::new("2"),
            DemoData { label: "Right".into(), ..Default::default() })),
    }
}

fn triple_workspace() -> PaneNode<DemoData> {
    PaneNode::Split {
        direction: SplitDirection::Horizontal,
        ratio: 0.33,
        first: Box::new(PaneNode::leaf_with_activity(PaneId::new("10"), ActivityId::new("1"),
            DemoData { label: "Files".into(), ..Default::default() })),
        second: Box::new(PaneNode::Split {
            direction: SplitDirection::Horizontal,
            ratio: 0.5,
            first: Box::new(PaneNode::leaf_with_activity(PaneId::new("11"), ActivityId::new("2"),
                DemoData { label: "Search".into(), ..Default::default() })),
            second: Box::new(PaneNode::leaf_with_activity(PaneId::new("12"), ActivityId::new("3"),
                DemoData { label: "Settings".into(), ..Default::default() })),
        }),
    }
}

fn stacked_workspace() -> PaneNode<DemoData> {
    PaneNode::Split {
        direction: SplitDirection::Vertical,
        ratio: 0.5,
        first: Box::new(PaneNode::leaf_with_activity(PaneId::new("20"), ActivityId::new("1"),
            DemoData { label: "Top".into(), ..Default::default() })),
        second: Box::new(PaneNode::leaf_with_activity(PaneId::new("21"), ActivityId::new("3"),
            DemoData { label: "Bottom".into(), show_files: false, ..Default::default() })),
    }
}

// ── App ──────────────────────────────────────────────────────────────────────

#[component]
fn App() -> impl IntoView {
    let workspaces = vec![
        Workspace { id: WorkspaceId("default".into()), name: "Default".into(), tree: default_workspace() },
        Workspace { id: WorkspaceId("triple".into()), name: "Triple".into(), tree: triple_workspace() },
        Workspace { id: WorkspaceId("stacked".into()), name: "Stacked".into(), tree: stacked_workspace() },
    ];
    let workspace_mgr = WorkspaceManager::new(workspaces, WorkspaceId("default".into()));

    // Theme defines the color palette
    provide_context(MullionTheme {
        bg: "#0e0e0e".into(),
        surface: "#111111".into(),
        border: "#1a1a1a".into(),
        accent: "#222222".into(),
        highlight: "#333333".into(),
        text: "#eeeeee".into(),
        text_muted: "#888888".into(),
        drop_indicator: "rgba(255,255,255,0.06)".into(),
    });

    provide_context(ActivityBarStyle {
        icon_opacity: "1".into(),
        icon_active_opacity: "1".into(),
        expanded_padding: "10px".into(),
        ..Default::default()
    });
    provide_context(SplitHandleStyle {
        thickness: "2px".into(),
        ..Default::default()
    });

    // Demo-specific style CSS
    let demo_css = [
        DemoLayoutStyle::default().to_css(),
        FooterTabStyle::default().to_css(),
        InputStyle::default().to_css(),
    ].join("\n");

    let on_event = move |event: PaneEvent<DemoData>| {
        let desc = match &event {
            PaneEvent::Split { target, new_id, direction, .. } => {
                format!("[mullion] Split {:?} -> {:?} ({:?})", target, new_id, direction)
            }
            PaneEvent::Closed { id, .. } => format!("[mullion] Closed {:?}", id),
            PaneEvent::Resized { split_key, ratio } => format!("[mullion] Resized split {:?} to {:.0}%", split_key, ratio * 100.0),
            PaneEvent::Moved { source, destination, edge } => format!("[mullion] Moved {:?} -> {:?} ({:?})", source, destination, edge),
            PaneEvent::DirectionChanged { pane, direction } => format!("[mullion] Dir {:?} -> {:?}", pane, direction),
            PaneEvent::ActivityChanged { pane, activity } => format!("[mullion] Activity {:?} -> {:?}", pane, activity),
            PaneEvent::TreeChanged { .. } => return,
        };
        web_sys::console::log_1(&desc.into());
    };

    view! {
        <style>{demo_css}</style>
        <MullionProvider
            initial_tree=default_workspace()
            categories=categories()
            on_event=on_event
            app_icon=ActivityIcon::Svg(outlined::ICON_APPS.into())
        >
            <DemoLayout workspace_mgr=workspace_mgr />
        </MullionProvider>
    }
}

#[component]
fn DemoLayout(workspace_mgr: WorkspaceManager<DemoData>) -> impl IntoView {
    let ctx = use_context::<MullionContext<DemoData>>()
        .expect("MullionContext provided by MullionProvider");

    let mgr = workspace_mgr.clone();
    let ctx_for_footer = ctx.clone();

    view! {
        <div class=DemoLayoutStyle::SCOPE>
            <div class=DemoLayoutStyle::CONTENT>
                <MullionPaneTree ctx=ctx />
            </div>
            <div class=DemoLayoutStyle::FOOTER>
                {move || {
                    let ws_list = mgr.list();
                    let current = mgr.active_id();
                    ws_list.into_iter().enumerate().map(|(i, ws)| {
                        let is_active = ws.id == current;
                        let class = if is_active {
                            FooterTabStyle::class(&[FooterTabModifier::Active])
                        } else {
                            FooterTabStyle::SCOPE.to_string()
                        };
                        let mgr = mgr.clone();
                        let ctx = ctx_for_footer.clone();
                        let ws_id = ws.id.clone();
                        view! {
                            <button class=class on:click=move |_| {
                                if let Some(tree) = mgr.switch_to(&ws_id) {
                                    ctx.set_tree(tree);
                                }
                            }>
                                {format!("{}", i + 1)}
                            </button>
                        }
                    }).collect::<Vec<_>>()
                }}
            </div>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <App /> });
}
