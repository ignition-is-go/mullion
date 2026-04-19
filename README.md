# mullion

A [Leptos](https://leptos.dev) component library for splittable panes with activity bars.

Named after the vertical bars between window panes in architecture.

## Features

- **Splittable panes** -- split horizontally or vertically, resize by dragging, close panes
- **Activity bar** -- collapsible categories with icons, expands on hover to show labels
- **Drag and drop** -- move panes between positions by dragging the app icon
- **Workspaces** -- named layouts you can switch between
- **Theming** -- all styling via Rust structs passed through `provide_context`, zero CSS required
- **Events** -- stream of pane events for persistence
- **Upstream signals** -- update the tree live from server queries
- **Pane data** -- generic consumer data per pane, filters which activities appear
- **String IDs** -- all IDs (pane, activity, category) are string-based for stable persistence

## Quick start

```rust
use leptos::prelude::*;
use mullion::*;

#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct MyData {
    project: String,
}

#[component]
fn App() -> impl IntoView {
    let categories = vec![
        Category {
            id: CategoryId::new("explorer"),
            name: "Explorer".into(),
            order: 0,
            icon: ActivityIcon::Svg("<svg>...</svg>".into()),
            color: "#75beff".into(),
            activities: vec![
                ActivityDef {
                    id: ActivityId::new("files"),
                    name: "Files".into(),
                    icon: ActivityIcon::Svg("<svg>...</svg>".into()),
                    filter: |_| true,
                    render: |_pid, data| {
                        view! { <p>"Files for " {move || data.get().project}</p> }.into_any()
                    },
                },
            ],
        },
    ];

    let tree = PaneNode::leaf_with_activity(
        PaneId::new("main"),
        ActivityId::new("files"),
        MyData { project: "my-app".into() },
    );

    view! {
        <MullionRoot
            initial_tree=tree
            categories=categories
            on_event=|_| {}
        />
    }
}
```

## Theming

All visual properties are controlled through theme structs provided via Leptos context. Set them before mounting `MullionProvider` or `MullionRoot`:

```rust
provide_context(ActivityBarTheme {
    width: "28px".into(),
    expanded_width: "150px".into(),
    icon_size: "14px".into(),
    background: "#111".into(),
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

provide_context(PaneTheme {
    background: "#111".into(),
    color: "#eee".into(),
});

provide_context(SplitHandleTheme {
    thickness: "2px".into(),
    color: "#1a1a1a".into(),
    hover_color: "#333".into(),
});

provide_context(MullionTheme {
    background: "#0e0e0e".into(),
});

provide_context(DropOverlayTheme {
    indicator_color: "rgba(255,255,255,0.06)".into(),
});
```

Active activity icons automatically take their category's color.

### Disabling hover-to-expand

By default the activity bar widens and reveals activity labels on hover. To
pin it to a fixed width, set `expanded_width` equal to `width`, zero the
`expanded_padding`, and set `label_hover_display` to `none`:

```rust
provide_context(ActivityBarStyle {
    width: "28px".into(),
    expanded_width: "28px".into(),
    expanded_padding: "0".into(),
    label_hover_display: "none".into(),
    ..Default::default()
});
```

## Components

| Component | Purpose |
|-----------|---------|
| `MullionRoot` | All-in-one: provides context and renders the pane tree |
| `MullionProvider` | Context-only provider, render children with full layout control |
| `MullionPaneTree` | Renders just the pane tree (use inside `MullionProvider`) |
| `WorkspaceSwitcher` | Batteries-included workspace tab bar |

## API

### MullionContext

Available via `use_context::<MullionContext<D>>()` inside a `MullionProvider`:

```rust
// Pane operations
ctx.split_pane(&pane_id, SplitDirection::Horizontal, PaneId::new("new-pane"), new_data);
ctx.close_pane(&pane_id);
ctx.resize_pane(&pane_id, 0.5);
ctx.move_pane(&source_id, &dest_id, DropEdge::Right);
ctx.change_split_direction(&pane_id, SplitDirection::Vertical);
ctx.set_active_activity(&pane_id, Some(ActivityId::new("files")));

// Pane data
ctx.update_pane_data(&pane_id, new_data);  // Update a single pane's data
ctx.pane_data(&pane_id);                   // Read a pane's data

// Read state
ctx.focused_pane.get()       // Option<PaneId> -- pane under mouse
ctx.dragging_pane.get()      // Option<PaneId> -- pane being dragged
ctx.pane_element(&pane_id)   // Option<HtmlElement> -- DOM ref for positioning
ctx.pane_rect(&pane_id)      // Option<DomRect> -- bounding rect

// Tree management
ctx.set_tree(new_tree);              // Replace entire tree (e.g. from server)
ctx.update_tree(|tree| { ... });     // Mutate the tree in place
```

### Workspaces

```rust
let mgr = WorkspaceManager::new(vec![
    Workspace { id: WorkspaceId("default".into()), name: "Default".into(), tree: my_tree },
], WorkspaceId("default".into()));

// Switch workspace
if let Some(tree) = mgr.switch_to(&WorkspaceId("other".into())) {
    ctx.set_tree(tree);
}
```

### Activity rendering

Activity components receive pane data via `ReadSignal<D>`:

```rust
ActivityDef {
    id: ActivityId::new("files"),
    name: "Files".into(),
    icon: ActivityIcon::Svg(md_icons::outlined::ICON_FOLDER.into()),
    filter: |d| d.show_files,
    render: |pane_id, data| {
        view! { <FilesPanel data=data /> }.into_any()
    },
}
```

## Running the demo

```sh
cd examples/demo
trunk serve
```

Open `http://localhost:8080`. Requires [Trunk](https://trunkrs.dev) and the `wasm32-unknown-unknown` target.

## License

MIT
