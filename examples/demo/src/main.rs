use leptos::prelude::*;
use md_icons::outlined;
use mullion::*;
use serde::{Deserialize, Serialize};

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
            id: CategoryId(0), name: "Explorer".into(), order: 0,
            icon: ActivityIcon::Svg(outlined::ICON_FOLDER.into()),
            color: "#75beff".into(),
            activities: vec![
                ActivityDef {
                    id: ActivityId(1), name: "Files".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_DESCRIPTION.into()),
                    filter: |d| d.show_files, render: |_pid, data| view! { <FilesActivity data=data /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId(2), name: "Open Editors".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_ARTICLE.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Open Editors" /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId(3), name: "Timeline".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_TIMELINE.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Timeline" /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId(4), name: "Outline".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_LIST.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Outline" /> }.into_any(),
                },
            ],
        },
        Category {
            id: CategoryId(1), name: "Edit".into(), order: 1,
            icon: ActivityIcon::Svg(outlined::ICON_EDIT_NOTE.into()),
            color: "#e8ab53".into(),
            activities: vec![
                ActivityDef {
                    id: ActivityId(5), name: "Search".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_SEARCH.into()),
                    filter: |d| d.show_search, render: |_pid, data| view! { <SearchActivity data=data /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId(6), name: "Replace".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_FIND_REPLACE.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Replace" /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId(7), name: "Bookmarks".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_BOOKMARKS.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Bookmarks" /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId(8), name: "Snippets".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_CODE.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Snippets" /> }.into_any(),
                },
            ],
        },
        Category {
            id: CategoryId(2), name: "Preferences".into(), order: 2,
            icon: ActivityIcon::Svg(outlined::ICON_SETTINGS.into()),
            color: "#c586c0".into(),
            activities: vec![
                ActivityDef {
                    id: ActivityId(9), name: "Settings".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_SETTINGS.into()),
                    filter: |d| d.show_settings, render: |_pid, _data| view! { <SettingsActivity /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId(10), name: "Themes".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_PALETTE.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Themes" /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId(11), name: "Keybindings".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_KEYBOARD.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Keybindings" /> }.into_any(),
                },
                ActivityDef {
                    id: ActivityId(12), name: "Extensions".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_EXTENSION.into()),
                    filter: |_| true, render: |_pid, _data| view! { <PlaceholderActivity name="Extensions" /> }.into_any(),
                },
            ],
        },
    ]
}

#[component]
fn FilesActivity(data: ReadSignal<DemoData>) -> impl IntoView {
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
fn SearchActivity(data: ReadSignal<DemoData>) -> impl IntoView {
    view! {
        <div class="activity-content">
            <h2>{move || data.get().label} " - Search"</h2>
            <p>"Type to search across files..."</p>
            <input type="text" placeholder="Search..."
                style="width: 100%; padding: 6px 8px; background: #3c3c3c; border: 1px solid #555; color: #ccc; border-radius: 3px; margin-top: 8px;" />
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

fn default_workspace() -> PaneNode<DemoData> {
    PaneNode::Split {
        direction: SplitDirection::Horizontal,
        ratio: 0.5,
        first: Box::new(PaneNode::leaf_with_activity(PaneId(1), ActivityId(1),
            DemoData { label: "Left".into(), ..Default::default() })),
        second: Box::new(PaneNode::leaf_with_activity(PaneId(2), ActivityId(2),
            DemoData { label: "Right".into(), ..Default::default() })),
    }
}

fn triple_workspace() -> PaneNode<DemoData> {
    PaneNode::Split {
        direction: SplitDirection::Horizontal,
        ratio: 0.33,
        first: Box::new(PaneNode::leaf_with_activity(PaneId(10), ActivityId(1),
            DemoData { label: "Files".into(), ..Default::default() })),
        second: Box::new(PaneNode::Split {
            direction: SplitDirection::Horizontal,
            ratio: 0.5,
            first: Box::new(PaneNode::leaf_with_activity(PaneId(11), ActivityId(2),
                DemoData { label: "Search".into(), ..Default::default() })),
            second: Box::new(PaneNode::leaf_with_activity(PaneId(12), ActivityId(3),
                DemoData { label: "Settings".into(), ..Default::default() })),
        }),
    }
}

fn stacked_workspace() -> PaneNode<DemoData> {
    PaneNode::Split {
        direction: SplitDirection::Vertical,
        ratio: 0.5,
        first: Box::new(PaneNode::leaf_with_activity(PaneId(20), ActivityId(1),
            DemoData { label: "Top".into(), ..Default::default() })),
        second: Box::new(PaneNode::leaf_with_activity(PaneId(21), ActivityId(3),
            DemoData { label: "Bottom".into(), show_files: false, ..Default::default() })),
    }
}

#[component]
fn App() -> impl IntoView {
    let workspaces = vec![
        Workspace { id: WorkspaceId("default".into()), name: "Default".into(), tree: default_workspace() },
        Workspace { id: WorkspaceId("triple".into()), name: "Triple".into(), tree: triple_workspace() },
        Workspace { id: WorkspaceId("stacked".into()), name: "Stacked".into(), tree: stacked_workspace() },
    ];
    let workspace_mgr = WorkspaceManager::new(workspaces, WorkspaceId("default".into()));

    provide_context(ActivityBarTheme {
        width: "28px".into(),
        expanded_width: "150px".into(),
        icon_size: "14px".into(),
        background: "#111111".into(),
        border: "1px solid #222".into(),
        border_radius: "0".into(),
        expanded_padding: "10px".into(),
        font_size: "11px".into(),
        icon_color: "#eee".into(),
        icon_stroke_color: "#eee".into(),
        icon_opacity: "1".into(),
        icon_active_opacity: "1".into(),
        category_border_width: "2px".into(),
    });
    provide_context(SplitHandleTheme {
        thickness: "2px".into(),
        color: "#1a1a1a".into(),
        hover_color: "#333".into(),
    });
    provide_context(PaneTheme {
        background: "#111111".into(),
        color: "#eee".into(),
    });
    provide_context(MullionTheme {
        background: "#0e0e0e".into(),
    });

    let on_event = move |event: PaneEvent<DemoData>| {
        let desc = match &event {
            PaneEvent::Split { target, new_id, direction, .. } => {
                format!("[mullion] Split {:?} -> {:?} ({:?})", target, new_id, direction)
            }
            PaneEvent::Closed { id, .. } => format!("[mullion] Closed {:?}", id),
            PaneEvent::Resized { pane, ratio } => format!("[mullion] Resized {:?} to {:.0}%", pane, ratio * 100.0),
            PaneEvent::Moved { source, destination, edge } => format!("[mullion] Moved {:?} -> {:?} ({:?})", source, destination, edge),
            PaneEvent::DirectionChanged { pane, direction } => format!("[mullion] Dir {:?} -> {:?}", pane, direction),
            PaneEvent::ActivityChanged { pane, activity } => format!("[mullion] Activity {:?} -> {:?}", pane, activity),
            PaneEvent::TreeChanged { .. } => return, // skip noisy tree snapshots
        };
        web_sys::console::log_1(&desc.into());
    };

    view! {
        <MullionProvider
            initial_tree=default_workspace()
            categories=categories()
            on_event=on_event
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
        <div style="display:flex;flex-direction:column;width:100vw;height:100vh;">
            <div style="flex:1;min-height:0;overflow:hidden;">
                <MullionPaneTree ctx=ctx />
            </div>
            <div style="display:flex;gap:1px;background:#0a0a0a;padding:2px 4px;border-top:1px solid #1a1a1a;">
                {move || {
                    let ws_list = mgr.list();
                    let current = mgr.active_id();
                    ws_list.into_iter().enumerate().map(|(i, ws)| {
                        let is_active = ws.id == current;
                        let bg = if is_active { "#222" } else { "transparent" };
                        let color = if is_active { "#aaa" } else { "#444" };
                        let style = format!(
                            "background:{};color:{};border:none;padding:2px 8px;font-size:11px;cursor:pointer;border-radius:2px;font-family:monospace",
                            bg, color
                        );
                        let mgr = mgr.clone();
                        let ctx = ctx_for_footer.clone();
                        let ws_id = ws.id.clone();
                        view! {
                            <button style={style} on:click=move |_| {
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
