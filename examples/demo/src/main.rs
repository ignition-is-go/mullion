use css_styled::{css, IntoCss, StyledComponent, StyledComponentBase};
use leptos::prelude::*;
use leptos_command_palette::{CommandPalette, CommandPaletteProvider};
use md_icons::outlined;
use mullion::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

fn items() -> Vec<ActivityNode<DemoData>> {
    vec![
        ActivityNode::Category(Category {
            id: CategoryId::new("0"),
            name: "Explorer".into(),
            icon: ActivityIcon::Svg(outlined::ICON_FOLDER.into()),
            color: "#75beff".into(),
            children: vec![
                ActivityNode::activity(ActivityDef {
                    id: ActivityId::new("1"),
                    name: "Files".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_DESCRIPTION.into()),
                    filter: |d| d.show_files,
                    render: |_pid, data| view! { <FilesActivity data=data /> }.into_any(),
                    // Showcase: custom header content beside the "Files" name —
                    // here, the pane's label, reactive to this pane's data.
                    header: Some(|_pid, data| {
                        view! { <span style="opacity:0.6">{move || data.get().label}</span> }
                            .into_any()
                    }),
                }),
                ActivityNode::activity(ActivityDef {
                    id: ActivityId::new("2"),
                    name: "Open Editors".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_ARTICLE.into()),
                    filter: |_| true,
                    render: |_pid, _data| view! { <OpenEditorsActivity /> }.into_any(),
                    header: None,
                }),
                ActivityNode::activity(ActivityDef {
                    id: ActivityId::new("3"),
                    name: "Timeline".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_TIMELINE.into()),
                    filter: |_| true,
                    render: |_pid, _data| view! { <TimelineActivity /> }.into_any(),
                    header: None,
                }),
                ActivityNode::activity(ActivityDef {
                    id: ActivityId::new("4"),
                    name: "Outline".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_LIST.into()),
                    filter: |_| true,
                    render: |_pid, _data| {
                        view! { <PlaceholderActivity name="Outline" /> }.into_any()
                    },
                    header: None,
                }),
            ],
        }),
        ActivityNode::Category(Category {
            id: CategoryId::new("1"),
            name: "Edit".into(),
            icon: ActivityIcon::Svg(outlined::ICON_EDIT_NOTE.into()),
            color: "#e8ab53".into(),
            children: vec![
                ActivityNode::activity(ActivityDef {
                    id: ActivityId::new("5"),
                    name: "Search".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_SEARCH.into()),
                    filter: |d| d.show_search,
                    render: |_pid, data| view! { <SearchActivity data=data /> }.into_any(),
                    header: None,
                }),
                ActivityNode::activity(ActivityDef {
                    id: ActivityId::new("6"),
                    name: "Replace".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_FIND_REPLACE.into()),
                    filter: |_| true,
                    render: |_pid, _data| {
                        view! { <PlaceholderActivity name="Replace" /> }.into_any()
                    },
                    header: None,
                }),
                ActivityNode::activity(ActivityDef {
                    id: ActivityId::new("7"),
                    name: "Bookmarks".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_BOOKMARKS.into()),
                    filter: |_| true,
                    render: |_pid, _data| {
                        view! { <PlaceholderActivity name="Bookmarks" /> }.into_any()
                    },
                    header: None,
                }),
                ActivityNode::activity(ActivityDef {
                    id: ActivityId::new("8"),
                    name: "Snippets".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_CODE.into()),
                    filter: |_| true,
                    render: |_pid, _data| {
                        view! { <PlaceholderActivity name="Snippets" /> }.into_any()
                    },
                    header: None,
                }),
            ],
        }),
        ActivityNode::Category(Category {
            id: CategoryId::new("2"),
            name: "Preferences".into(),
            icon: ActivityIcon::Svg(outlined::ICON_SETTINGS.into()),
            color: "#c586c0".into(),
            children: vec![
                ActivityNode::activity(ActivityDef {
                    id: ActivityId::new("10"),
                    name: "Themes".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_PALETTE.into()),
                    filter: |_| true,
                    render: |_pid, _data| {
                        view! { <PlaceholderActivity name="Themes" /> }.into_any()
                    },
                    header: None,
                }),
                // A category nested inside a category — arbitrary depth. Its
                // children inherit *this* colour for their active state, not
                // Preferences'.
                ActivityNode::Category(Category {
                    id: CategoryId::new("3"),
                    name: "Advanced".into(),
                    icon: ActivityIcon::Svg(outlined::ICON_TUNE.into()),
                    color: "#e8ab53".into(),
                    children: vec![
                        ActivityNode::activity(ActivityDef {
                            id: ActivityId::new("11"),
                            name: "Keybindings".into(),
                            icon: ActivityIcon::Svg(outlined::ICON_KEYBOARD.into()),
                            filter: |_| true,
                            render: |_pid, _data| view! { <KeybindingsActivity /> }.into_any(),
                            header: None,
                        }),
                        ActivityNode::activity(ActivityDef {
                            id: ActivityId::new("12"),
                            name: "Extensions".into(),
                            icon: ActivityIcon::Svg(outlined::ICON_EXTENSION.into()),
                            filter: |_| true,
                            render: |_pid, _data| {
                                view! { <PlaceholderActivity name="Extensions" /> }.into_any()
                            },
                            header: None,
                        }),
                    ],
                }),
            ],
        }),
    ]
}

/// The bottom activity group — anchored to the foot of the bar rather than
/// trailing the last category. Same shape as `items()`: nesting, categories and
/// drag-to-create all work here identically.
fn bottom_items() -> Vec<ActivityNode<DemoData>> {
    vec![ActivityNode::activity(ActivityDef {
        id: ActivityId::new("9"),
        name: "Settings".into(),
        icon: ActivityIcon::Svg(outlined::ICON_SETTINGS.into()),
        filter: |d| d.show_settings,
        render: |pid, _data| view! { <SettingsActivity pane_id=pid /> }.into_any(),
        header: None,
    })]
}

// ── Activity content views ───────────────────────────────────────────────────

#[component]
fn FilesActivity(data: Signal<DemoData>) -> impl IntoView {
    let files = [
        ("src/components/pane_view.rs", "M", "modified"),
        ("src/components/mullion_root.rs", "M", "modified"),
        ("src/commands.rs", "M", "modified"),
        ("src/context.rs", "M", "modified"),
        ("src/focus.rs", "M", "modified"),
        ("src/settings.rs", "M", "modified"),
        ("src/tree.rs", "", "muted"),
        ("src/theme.rs", "", "muted"),
        ("examples/demo/src/main.rs", "M", "modified"),
        ("examples/demo/index.html", "M", "modified"),
        ("tests/focus_visual.rs", "A", "added"),
        ("Cargo.toml", "", "muted"),
        ("README.md", "M", "modified"),
    ];
    view! {
        <div class="activity-content">
            <h2>{move || data.get().label} " - Files"</h2>
            <p>"Working tree for the focused-pane presentation pass."</p>
            <div class="section-label">"Changes"</div>
            <div>
                {files.into_iter().map(|(path, status, tone)| view! {
                    <div class="file-item file-row">
                        <span class="file-path">{path}</span>
                        <span class=format!("file-status {tone}")>{status}</span>
                    </div>
                }).collect::<Vec<_>>()}
            </div>
            <div class="summary-grid">
                <div class="summary-card"><span class="summary-value">"13"</span><span class="summary-label">"Files"</span></div>
                <div class="summary-card"><span class="summary-value">"76"</span><span class="summary-label">"Tests"</span></div>
                <div class="summary-card"><span class="summary-value success">"✓"</span><span class="summary-label">"WASM"</span></div>
            </div>
        </div>
    }
}

#[component]
fn OpenEditorsActivity() -> impl IntoView {
    let editors = [
        ("pane_view.rs", "src/components", "428 lines", "#75beff"),
        ("mullion_root.rs", "src/components", "326 lines", "#c586c0"),
        ("main.rs", "examples/demo/src", "641 lines", "#e8ab53"),
        ("README.md", "documentation", "472 lines", "#73c991"),
    ];

    view! {
        <div class="activity-content">
            <h2>"Open Editors"</h2>
            <p>"Four files are open across the focus-state iteration."</p>
            <div class="section-label">"Working set"</div>
            <div class="editor-list">
                {editors.into_iter().map(|(name, path, meta, color)| view! {
                    <div class="editor-card">
                        <span class="editor-accent" style=format!("background:{color}")></span>
                        <span>
                            <span class="editor-name">{name}</span>
                            <span class="editor-path">{path}</span>
                        </span>
                        <span class="editor-meta">{meta}</span>
                    </div>
                }).collect::<Vec<_>>()}
            </div>
            <div class="section-label">"Current selection"</div>
            <div class="code-preview"><span class="code-keyword">"let"</span> " focused = "<span class="code-type">"show_focus_indicator"</span>"\n    && focused_pane.get() == pane_id;\n\n"<span class="code-keyword">"view!"</span>" { <PaneView focused=focused /> }"</div>
        </div>
    }
}

#[component]
fn TimelineActivity() -> impl IntoView {
    let events = [
        (
            "09:45",
            "Release demo built",
            "Optimized WASM bundle generated successfully.",
            "success",
        ),
        (
            "09:42",
            "Focus presentation updated",
            "One-pixel accent and geometry-aware internal edges.",
            "info",
        ),
        (
            "09:38",
            "Test suite passed",
            "76 unit tests and two documentation tests completed.",
            "success",
        ),
        (
            "09:31",
            "Compatibility default restored",
            "Existing consumers retain hover focus and unchanged chrome.",
            "modified",
        ),
        (
            "09:18",
            "Command catalog registered",
            "Focus, split, move, resize, layout, and zoom actions available.",
            "info",
        ),
        (
            "09:05",
            "Workspace opened",
            "Three-pane main-and-stack layout restored from session state.",
            "muted",
        ),
    ];

    view! {
        <div class="activity-content">
            <h2>"Timeline"</h2>
            <p>"Recent activity from the Mullion design session."</p>
            <div class="section-label">"Today"</div>
            <div class="timeline-list">
                {events.into_iter().map(|(time, title, detail, tone)| view! {
                    <div class="timeline-row">
                        <span class="timeline-time">{time}</span>
                        <span class=format!("timeline-rail {tone}")><span class="timeline-dot"></span></span>
                        <span>
                            <span class="timeline-title">{title}</span>
                            <span class="timeline-detail">{detail}</span>
                        </span>
                    </div>
                }).collect::<Vec<_>>()}
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
fn KeybindingsActivity() -> impl IntoView {
    let bindings = [
        ("Alt + Arrow", "Focus in that direction"),
        ("Alt + Shift + Arrow", "Move focused pane"),
        ("Ctrl + Shift + Arrow", "Swap with a neighbor"),
        ("Ctrl + Alt + Arrow", "Resize toward a boundary"),
        ("Ctrl + Alt + Shift + →/↓", "New pane right / down"),
        ("Ctrl + Shift + Backspace", "Close focused pane"),
        ("Ctrl + Shift + Enter", "Toggle focused-pane zoom"),
        ("Ctrl + Alt + =", "Balance splits"),
        ("Ctrl + Alt + 1…5", "Apply a standard layout"),
    ];

    view! {
        <div class="activity-content">
            <h2>"Mullion keybindings"</h2>
            <p style="margin-bottom:12px">"Direct shortcuts—no leader or pane mode."</p>
            <div style="display:grid;grid-template-columns:max-content 1fr;gap:7px 12px;align-items:center;font-size:12px">
                {bindings.into_iter().map(|(keys, action)| view! {
                    <kbd style="padding:2px 6px;border:1px solid var(--ml-highlight);border-radius:4px;background:var(--ml-accent);font-family:monospace;color:var(--ml-text)">{keys}</kbd>
                    <span style="color:var(--ml-text-muted)">{action}</span>
                }).collect::<Vec<_>>()}
            </div>
            <p style="margin-top:14px">"Use Ctrl/⌘+K to browse every Mullion command."</p>
        </div>
    }
}

#[component]
fn SettingsActivity(pane_id: PaneId) -> impl IntoView {
    // This is deliberately demo-owned UI. It pulls Mullion's typed descriptor
    // from context just as a consumer's existing settings page/registry would.
    let setting = expect_context::<MullionSettings>().focus_behavior_setting();
    let group_name = format!("{}-{}", setting.id(), pane_id.0);
    let options = setting
        .options()
        .iter()
        .copied()
        .map(|option| {
            let current = setting.clone();
            let update = setting.clone();
            let value = *option.value();
            view! {
                <label style="display:grid;grid-template-columns:auto 1fr;gap:10px;align-items:start;padding:10px 12px;border:1px solid var(--ml-highlight);border-radius:5px;cursor:pointer">
                    <input
                        type="radio"
                        name=group_name.clone()
                        prop:checked=move || current.get() == value
                        on:change=move |_| update.set(value)
                        style="margin-top:2px;accent-color:var(--ml-primary,#00a4ef)"
                    />
                    <span>
                        <strong style="display:block;font-size:13px;color:var(--ml-text)">{option.label()}</strong>
                        <span style="display:block;margin-top:3px;font-size:12px;line-height:1.4;color:var(--ml-text-muted)">{option.description()}</span>
                    </span>
                </label>
            }
        })
        .collect::<Vec<_>>();

    view! {
        <div class="activity-content">
            <h2>{setting.label()}</h2>
            <p>{setting.description()}</p>
            <div style="display:grid;gap:8px;margin-top:14px;max-width:480px">
                {options}
            </div>
            <p style="margin-top:14px;color:var(--ml-text-muted)">
                "This control is rendered and persisted by the demo app from Mullion's headless setting descriptor."
            </p>
        </div>
    }
}

// ── Workspaces ───────────────────────────────────────────────────────────────

fn default_workspace() -> PaneNode<DemoData> {
    PaneNode::Split {
        direction: SplitDirection::Horizontal,
        ratio: 0.4,
        first: Box::new(PaneNode::leaf_with_activity(
            PaneId::new("1"),
            ActivityId::new("1"),
            DemoData {
                label: "Left".into(),
                ..Default::default()
            },
        )),
        second: Box::new(PaneNode::Split {
            direction: SplitDirection::Vertical,
            ratio: 0.5,
            first: Box::new(PaneNode::leaf_with_activity(
                PaneId::new("2"),
                ActivityId::new("2"),
                DemoData {
                    label: "Right Top".into(),
                    ..Default::default()
                },
            )),
            second: Box::new(PaneNode::leaf_with_activity(
                PaneId::new("3"),
                ActivityId::new("3"),
                DemoData {
                    label: "Right Bottom".into(),
                    ..Default::default()
                },
            )),
        }),
    }
}

fn triple_workspace() -> PaneNode<DemoData> {
    PaneNode::Split {
        direction: SplitDirection::Horizontal,
        ratio: 0.33,
        first: Box::new(PaneNode::leaf_with_activity(
            PaneId::new("10"),
            ActivityId::new("1"),
            DemoData {
                label: "Files".into(),
                ..Default::default()
            },
        )),
        second: Box::new(PaneNode::Split {
            direction: SplitDirection::Horizontal,
            ratio: 0.5,
            first: Box::new(PaneNode::leaf_with_activity(
                PaneId::new("11"),
                ActivityId::new("2"),
                DemoData {
                    label: "Search".into(),
                    ..Default::default()
                },
            )),
            second: Box::new(PaneNode::leaf_with_activity(
                PaneId::new("12"),
                ActivityId::new("3"),
                DemoData {
                    label: "Settings".into(),
                    ..Default::default()
                },
            )),
        }),
    }
}

fn stacked_workspace() -> PaneNode<DemoData> {
    PaneNode::Split {
        direction: SplitDirection::Vertical,
        ratio: 0.5,
        first: Box::new(PaneNode::leaf_with_activity(
            PaneId::new("20"),
            ActivityId::new("1"),
            DemoData {
                label: "Top".into(),
                ..Default::default()
            },
        )),
        second: Box::new(PaneNode::leaf_with_activity(
            PaneId::new("21"),
            ActivityId::new("3"),
            DemoData {
                label: "Bottom".into(),
                show_files: false,
                ..Default::default()
            },
        )),
    }
}

// ── App ──────────────────────────────────────────────────────────────────────

const FOCUS_BEHAVIOR_STORAGE_KEY: &str = "mullion-demo.focus-behavior";

fn load_focus_behavior() -> PaneFocusBehavior {
    web_sys::window()
        .and_then(|window| window.local_storage().ok().flatten())
        .and_then(|storage| storage.get_item(FOCUS_BEHAVIOR_STORAGE_KEY).ok().flatten())
        .and_then(|value| serde_json::from_str(&value).ok())
        // The library preserves legacy hover behavior by default. This muxer
        // demo deliberately opts into durable click focus.
        .unwrap_or(PaneFocusBehavior::Click)
}

fn store_focus_behavior(focus_behavior: PaneFocusBehavior) {
    let Some(storage) = web_sys::window().and_then(|window| window.local_storage().ok().flatten())
    else {
        return;
    };
    let Ok(value) = serde_json::to_string(&focus_behavior) else {
        return;
    };
    let _ = storage.set_item(FOCUS_BEHAVIOR_STORAGE_KEY, &value);
}

#[component]
fn App() -> impl IntoView {
    let workspaces = vec![
        Workspace {
            id: WorkspaceId("default".into()),
            name: "Default".into(),
            tree: default_workspace(),
        },
        Workspace {
            id: WorkspaceId("triple".into()),
            name: "Triple".into(),
            tree: triple_workspace(),
        },
        Workspace {
            id: WorkspaceId("stacked".into()),
            name: "Stacked".into(),
            tree: stacked_workspace(),
        },
    ];
    let workspace_mgr = WorkspaceManager::new(workspaces, WorkspaceId("default".into()));

    // The host owns the setting and its persistence. Mullion receives a
    // controlled handle, which the demo's existing Settings activity also
    // consumes to present the live value and setter.
    let focus_behavior = RwSignal::new(load_focus_behavior());
    let mullion_settings = MullionSettings::controlled(focus_behavior, move |next| {
        focus_behavior.set(next);
        store_focus_behavior(next);
    });

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
    ]
    .join("\n");

    let on_event = move |event: PaneEvent<DemoData>| {
        let desc = match &event {
            PaneEvent::Split {
                target,
                new_id,
                direction,
                ..
            } => {
                format!(
                    "[mullion] Split {:?} -> {:?} ({:?})",
                    target, new_id, direction
                )
            }
            PaneEvent::Closed { id, .. } => format!("[mullion] Closed {:?}", id),
            PaneEvent::Resized { split_key, ratio } => format!(
                "[mullion] Resized split {:?} to {:.0}%",
                split_key,
                ratio * 100.0
            ),
            PaneEvent::Moved {
                source,
                destination,
                edge,
            } => format!(
                "[mullion] Moved {:?} -> {:?} ({:?})",
                source, destination, edge
            ),
            PaneEvent::DirectionChanged { pane, direction } => {
                format!("[mullion] Dir {:?} -> {:?}", pane, direction)
            }
            PaneEvent::ActivityChanged { pane, activity } => {
                format!("[mullion] Activity {:?} -> {:?}", pane, activity)
            }
            PaneEvent::ActivityDropped {
                activity,
                destination,
                edge,
                new_id,
                ..
            } => {
                format!(
                    "[mullion] Dropped {:?} at {:?} of {:?} -> new pane {:?}",
                    activity, edge, destination, new_id
                )
            }
            PaneEvent::TreeChanged { .. } => return,
        };
        web_sys::console::log_1(&desc.into());
    };

    // Drop-to-create: dragging an activity out of the bar asks the host to mint
    // the pane, since only the host can allocate an id and build the pane's
    // data. A persisted app would create its pane entity here; the demo just
    // makes up an id and labels the pane after the activity it came from.
    let new_pane: PaneFactory<DemoData> = Arc::new(|activity, destination, edge| {
        let id = PaneId::new(format!(
            "drop-{:.0}",
            web_sys::js_sys::Math::random() * 1e12
        ));
        web_sys::console::log_1(
            &format!(
                "[demo] minting {:?} for {:?} ({:?} of {:?})",
                id, activity, edge, destination
            )
            .into(),
        );
        // Defaults leave every filter flag on, so the dropped activity is
        // guaranteed to pass its own `filter` in the new pane.
        Some((
            id,
            DemoData {
                label: format!("Activity {}", activity.0),
                ..Default::default()
            },
        ))
    });

    // Host slots either side of the bottom activity group. Hairlines here, but
    // the signature is the same as `pane_accessory`, so a host can render
    // anything per pane — a session indicator, a status dot, a divider.
    let rule = |color: &'static str| -> PaneAccessory {
        Arc::new(move |_pane_id| {
            view! { <div style=format!("height:1px;margin:4px 6px;background:{color}") /> }
                .into_any()
        })
    };
    let bottom_leading = rule("var(--ml-border)");
    let bottom_trailing = rule("var(--ml-border)");

    view! {
        <CommandPaletteProvider>
            <CommandPalette />
            <style>{demo_css}</style>
            <MullionProvider
                initial_tree=default_workspace()
                items=items()
                bottom_items=bottom_items()
                bottom_leading=bottom_leading
                bottom_trailing=bottom_trailing
                on_event=on_event
                app_icon=ActivityIcon::Svg(outlined::ICON_APPS.into())
                new_pane=new_pane
                settings=mullion_settings
                show_focus_indicator=true
                unfocused_pane_opacity=0.75
            >
                <DemoLayout workspace_mgr=workspace_mgr />
            </MullionProvider>
        </CommandPaletteProvider>
    }
}

#[component]
fn DemoLayout(workspace_mgr: WorkspaceManager<DemoData>) -> impl IntoView {
    let ctx = use_context::<MullionContext<DemoData>>()
        .expect("MullionContext provided by MullionProvider");

    let mgr = workspace_mgr.clone();
    let ctx_for_footer = ctx.clone();
    let commands =
        MullionCommands::new(ctx.clone()).with_split_factory_fn(|_focused, direction, data| {
            let id = PaneId::new(format!("mux-{:.0}", web_sys::js_sys::Math::random() * 1e12));
            let axis = match direction {
                SplitDirection::Horizontal => "horizontal",
                SplitDirection::Vertical => "vertical",
            };
            Some((
                id,
                DemoData {
                    label: format!("{} ({axis} split)", data.label),
                    ..data.clone()
                },
            ))
        });

    view! {
        <MullionKeybindings commands=commands.clone() />
        <MullionCommandPalette commands=commands />
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
                <span style="margin-left:auto;padding:2px 6px;color:var(--ml-text-muted);font:11px monospace">
                    "Alt+Arrow · focus   Ctrl/⌘+K · all commands"
                </span>
            </div>
        </div>
    }
}

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| view! { <App /> });
}
