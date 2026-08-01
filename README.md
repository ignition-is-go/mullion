# mullion

[![CI](https://github.com/ignition-is-go/mullion/actions/workflows/ci.yml/badge.svg)](https://github.com/ignition-is-go/mullion/actions/workflows/ci.yml)

A [Leptos](https://leptos.dev) component library for splittable panes with activity bars.

Named after the vertical bars between window panes in architecture.

## Features

- **Splittable panes** -- split horizontally or vertically, resize by dragging, close panes
- **Activity bar** -- place it on any pane edge, with collapsible categories and hover labels
- **Drag and drop** -- move panes between positions by dragging the app icon
- **Focused panes** -- durable focus with configurable hover or click acquisition
- **Pane commands** -- navigate, split, close, move, swap, resize, rotate, balance, lay out, and zoom panes
- **Pane keybindings** -- opt-in, customizable direct chords using familiar modifier combinations
- **Command palettes** -- optional `leptos-command-palette` command adapter
- **Workspaces** -- named layouts you can switch between
- **Theming** -- all styling via Rust structs passed through `provide_context`, zero CSS required
- **Events** -- stream of pane events for persistence
- **Upstream signals** -- update the tree live from server queries
- **Pane data** -- generic consumer data per pane, filters which activities appear
- **String IDs** -- all IDs (pane, activity, category) are string-based for stable persistence

## CI and releases

Pull requests and pushes to `main` run library formatting, native and wasm
checks, Clippy, tests, and a production build of the standalone demo on the
ephemeral GitHub-hosted Ubuntu runner. After CI succeeds on `main`, cargo-flux
derives the next version from Conventional Commits, stamps the manifest, and
publishes a version tag and GitHub Release. Mullion is consumed from Git and is
not published to crates.io, where the package name is owned by another project.

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

### Activity bar behavior

Interaction semantics that can't be expressed as a CSS variable live on
`ActivityBarBehavior`. Provide it via context before mounting the root:

```rust
provide_context(ActivityBarBehavior {
    hover_expand: false,   // pin the bar at its collapsed width; default: true
});
```

### Activity bar edge

Activity bars are vertical on the left by default. Set the optional edge prop
on `MullionRoot` or `MullionProvider` to place one on any pane edge:

```rust
view! {
    <MullionRoot
        initial_tree=tree
        items=items
        activity_bar_edge=ActivityBarEdge::Top
        on_event=|_| {}
    />
}
```

`Left` and `Right` arrange items vertically; `Top` and `Bottom` arrange them
horizontally. On horizontal edges the primary group flows from the left and
`bottom_items` becomes the trailing group on the right. Hover expansion applies
to one item at a time, and an auto-hidden bar reveals from its configured edge.

## Focus and pane commands

The first pane starts focused. Focus is durable state in `ctx.focused_pane`, is
marked by a high-contrast accent on the internal separators surrounding that
pane, and drives every focus-relative command. Root-container edges are omitted,
so the marker traces only the boundaries that distinguish the focused pane from
its neighbors. A pinned activity bar gets its own visual lane: the focus frame
is inset to the bar's content edge and painted below the bar, so category and
active-activity colors never compete with pane focus. Choose how pointer
interaction changes focus:

```rust
let app_focus_behavior = RwSignal::new(load_focus_behavior());
let mullion_settings = MullionSettings::controlled(
    app_focus_behavior,
    move |next| {
        app_focus_behavior.set(next);
        save_focus_behavior(next); // optional host persistence
    },
);

view! {
    <MullionProvider
        initial_tree=tree
        items=items
        settings=mullion_settings
        on_event=|_| {}
    >
        <AppLayout />
    </MullionProvider>
}
```

`Click` is the default and keeps focus on the last pane pressed. `Hover` is an
opt-in focus-follows-pointer mode. Programmatic focus works in either mode.
Set `--ml-focus-color` and `--ml-focus-width` on any Mullion ancestor to theme
the indicator. The color falls back to `--ml-primary`, then `#00a4ef`; width
defaults to `2px`. These standalone variables avoid adding fields to the public
`MullionTheme` struct.

### Settings integration

`MullionSettings` is a headless, controlled handle—not a settings page and not
a persistence layer. The consuming application keeps its own source of truth
and passes its live signal plus setter to `MullionSettings::controlled`. The
same handle can then be cloned into the app's existing settings page or generic
settings registry:

```rust
let pane_focus = mullion_settings.focus_behavior_setting();

settings_registry.register_select(
    pane_focus.id(),
    pane_focus.label(),
    pane_focus.description(),
    pane_focus.options(),
    pane_focus.value_signal(),
    move |next| pane_focus.set(next),
);
```

The registry call above is schematic—the registry belongs to the host—but every
piece it needs is supplied by `MullionSetting<PaneFocusBehavior>`: the stable id
`mullion.focus_behavior`, label, description, typed options, reactive value, and
setter. `MullionProvider` also provides its `MullionSettings` handle through
Leptos context, which is convenient for an app-owned settings activity rendered
inside Mullion.

For applications without a settings system, use
`MullionSettings::local(PaneFocusBehavior::Click)` or simply omit the `settings`
prop to get a local click-to-focus default. The runtime handle itself is not
serialized; persist the serializable `PaneFocusBehavior` in the application's
existing configuration model.

`MullionCommands<D>` is the dependency-free command dispatcher. Split commands
require a host factory because Mullion cannot invent application ids or data:

```rust
#[component]
fn PaneControls() -> impl IntoView {
    let ctx = expect_context::<MullionContext<MyData>>();
    let commands = MullionCommands::new(ctx)
        .with_split_factory_fn(|focused, direction, data| {
            Some((allocate_pane_id(focused, direction), data.clone()))
        });

    view! {
        <MullionKeybindings commands=commands />
    }
}
```

The command catalog covers directional/next/previous/indexed focus; horizontal
and vertical split; close; directional move, swap, and resize; parent split
orientation; balancing and rotation; even-horizontal, even-vertical,
main-horizontal, main-vertical, and tiled layouts; and focused-pane zoom.
Commands return `PaneCommandResult`, so an app can surface cases such as a
missing neighbor, refused split, or attempt to close the last pane.

### Default pane keymap

`MullionKeybindings` is opt-in and renders no DOM. Its default
`MullionKeymap::mullion()` uses direct modifier chords; there is no leader or
pane mode. Directional commands share the arrow keys and differ by modifiers,
while less frequent layout commands use mnemonic keys. Editable elements are
ignored.

Build a direct custom map with `MullionKeymap::unprefixed`, or a leader-based
map with `MullionKeymap::new`. Add bindings with `bind`, `bind_sequence`,
`with_binding`, and `with_sequence`. Focus behavior, commands, chords, bindings,
and keymaps all implement Serde traits, so applications can store this
interaction setup as configuration. `MullionKeymap::tmux()` remains an explicit
preset for applications whose users already expect tmux.

| Shortcut | Command |
|---|---|
| `Alt+Arrow` | Focus the pane in that direction |
| `Alt+PageDown` / `Alt+PageUp` | Focus next / previous pane |
| `Alt+Home` / `Alt+End` | Focus first / last pane |
| `Alt+1`…`Alt+9` | Focus a pane by number |
| `Alt+Shift+Arrow` | Move the focused pane |
| `Ctrl+Shift+Arrow` | Swap with a directional neighbor |
| `Ctrl+Alt+Arrow` | Resize toward that boundary |
| `Ctrl+Alt+Shift+Right` / `Down` | Create a pane to the right / below |
| `Ctrl+Shift+Backspace` | Close the focused pane |
| `Ctrl+Shift+Enter` | Toggle focused-pane zoom |
| `Ctrl+Shift+PageUp` / `PageDown` | Swap with previous / next pane |
| `Ctrl+Alt+H` / `V` / `O` | Orient side-by-side / stacked / toggle |
| `Ctrl+Alt+=` | Balance every split |
| `Ctrl+Alt+[` / `]` | Rotate backward / forward |
| `Ctrl+Alt+1`…`Ctrl+Alt+5` | Apply a standard layout |

### Command-palette integration

Enable the optional adapter:

```toml
mullion = { git = "https://github.com/ignition-is-go/mullion.git", features = ["command-palette"] }
```

Mount `<MullionCommandPalette commands=commands />` under a
`leptos_command_palette::CommandPaletteProvider` to register and automatically
clean up Mullion's catalog. Apps with their own registration lifecycle can call
`mullion_palette_commands(commands)` and merge the returned `Vec<Command>`
instead. The live “Focus Pane…” submenu is generated from the current layout;
split entries are omitted until the dispatcher has a split factory.

## Components

| Component | Purpose |
|-----------|---------|
| `MullionRoot` | All-in-one: provides context and renders the pane tree |
| `MullionProvider` | Context-only provider, render children with full layout control |
| `MullionPaneTree` | Renders just the pane tree (use inside `MullionProvider`) |
| `MullionKeybindings` | Opt-in global listener for a prefix keymap |
| `MullionCommandPalette` | Feature-gated command registration adapter |
| `WorkspaceSwitcher` | Batteries-included workspace tab bar |
| `MullionOverlay` | Portals content above all chrome, for modals that must escape their pane |

## Stacking: chrome, content, and overlays

Mullion paints its chrome in a small, fixed z-band inside the pane tree:

| z-index | element |
|---------|---------|
| 5 | split handles |
| 10 | activity bar panel |
| 20 | drop overlay (drag feedback) |

Activity content is confined below that band: the pane's content column is
`isolation: isolate`, so it is its own stacking context. Whatever z-index an
activity uses is resolved *within its own pane* — it stays meaningful relative
to the activity's own elements, and can never compete with chrome or with
another pane.

This means an activity cannot escape its pane by picking a bigger number. Two
consequences worth knowing:

- **In-pane popovers and dropdowns are unaffected.** The pane's content slot is
  already `overflow: hidden`, so they were always clipped to the pane.
- **Full-screen modals, command palettes, and anything else that must cover the
  app has to leave the pane** — use `MullionOverlay`. A `position: fixed` element
  inside activity content still covers the viewport geometrically, but it is
  painted inside its pane's stacking context, so a neighbouring pane's content
  will cover it in a split layout.

### MullionOverlay

```rust
use mullion::{MullionOverlay, OverlayLevel};

view! {
    <Show when=move || open.get()>
        <MullionOverlay
            backdrop=true
            center=true
            on_click_outside=Callback::new(move |_| open.set(false))
        >
            <MyDialog />
        </MullionOverlay>
    </Show>
}
```

The first overlay to mount lazily creates a single `<div id="mullion-overlay-root">`
on `document.body` — `position: fixed; inset: 0; z-index: var(--ml-overlay-z, 10000)`
and `pointer-events: none`, so an idle overlay layer never eats clicks. Each
overlay is its own stacking context inside that root, so an overlay's internal
z-indexes keep working relative to each other without leaking.

| Prop | Default | Purpose |
|------|---------|---------|
| `level` | `OverlayLevel::Modal` | Tier against *other* overlays: `Modal` < `Toast` < `Drag`. Every tier is above all chrome. `Drag` is reserved for mullion. |
| `backdrop` | `false` | Dimming scrim behind the children |
| `backdrop_color` | `var(--ml-scrim, rgba(0,0,0,0.5))` | Define `--ml-scrim` to theme every backdrop at once |
| `center` | `false` | Center the children in the viewport |
| `on_click_outside` | — | Click-to-dismiss. Fires only for clicks that land on the overlay's own box, so it does **not** fire if your children fill the overlay themselves (e.g. your own `inset: 0` wrapper) — use `center`, or handle dismissal yourself. |
| `click_through` | `false` | Let pointer events reach the app behind the overlay. Content must then set `pointer-events: auto`. |

Children are a `ChildrenFn`, so any captured state must be `Fn`-safe — copy
signals in, or park non-`Copy` values in a `StoredValue`.

Reactive context is preserved across the portal: `use_context`, signals, and
effects behave exactly as they do inline.

## API

### MullionContext

Available via `use_context::<MullionContext<D>>()` inside a `MullionProvider`:

```rust
// Pane operations
ctx.split_pane(&pane_id, SplitDirection::Horizontal, PaneId::new("new-pane"), new_data);
ctx.close_pane(&pane_id);
ctx.resize_split(&split_key, 0.5);  // split_key = first leaf id under the split's `second` subtree
ctx.move_pane(&source_id, &dest_id, DropEdge::Right);
ctx.change_split_direction(&pane_id, SplitDirection::Vertical);
ctx.swap_panes(&first_id, &second_id);
ctx.rotate_panes(PaneRotation::Forward);
ctx.balance_splits();
ctx.apply_layout(PaneLayout::Tiled);
ctx.resize_pane_toward(&pane_id, PaneDirection::Left, 0.05);
ctx.set_active_activity(&pane_id, Some(ActivityId::new("files")));

// Focus and view state
ctx.focus_pane(&pane_id);
ctx.focus_neighbor(PaneDirection::Right);
ctx.cycle_focus(1);
ctx.toggle_zoom();

// Pane data
ctx.update_pane_data(&pane_id, new_data);  // Update a single pane's data
ctx.pane_data(&pane_id);                   // Read a pane's data

// Read state
ctx.focused_pane.get()       // Option<PaneId> -- command target
ctx.zoomed_pane.get()        // Option<PaneId> -- full-viewport pane
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
