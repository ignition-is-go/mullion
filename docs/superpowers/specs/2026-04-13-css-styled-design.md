# css-styled: Typed, Spec-Validated CSS for Rust Component Libraries

## Problem

Building themed component libraries in Rust/Leptos means scattering CSS custom properties across `.css` files and Rust code with no type checking, no validation, and no connection between the theme values a consumer sets and the CSS that uses them. Existing Leptos styling solutions (Stylers, Stylance, Turf, RCSS, etc.) handle scoped CSS authoring but none provide typed theme structs, compile-time CSS spec validation, or a struct-to-CSS pipeline.

## Solution

A standalone Rust crate (`css-styled`) where a single struct defines a component's entire styling contract: customizable theme values, class names, modifiers, and structural CSS. Everything is validated against the CSS spec at compile time. The consumer configures fields on the struct. The crate produces aggregated CSS for a single `<style>` tag at the app root.

## Workspace Structure

```
css-styled/
├── Cargo.toml                  # workspace root
├── css-spec-data/
│   ├── Cargo.toml
│   ├── build.rs                # fetches/parses @webref/css, generates lookup tables
│   ├── data/                   # cached spec data (checked into git for offline builds)
│   └── src/
│       └── lib.rs              # PropertySpec, property(), validate_value()
├── css-styled-derive/
│   ├── Cargo.toml              # proc-macro = true, depends on css-spec-data
│   └── src/
│       └── lib.rs              # #[derive(StyledComponent)], css!() macro
└── css-styled/
    ├── Cargo.toml              # depends on css-styled-derive, css-spec-data; optional "leptos" feature
    └── src/
        ├── lib.rs              # re-exports derive, traits, leptos integration
        ├── traits.rs           # IntoCss, StyledComponentBase trait definitions
        └── validate.rs         # runtime value validation
```

Consumer dependency:

```toml
css-styled = { version = "0.1", features = ["leptos"] }
```

## Crate: css-spec-data

Generated from the `@webref/css` machine-readable W3C spec data at build time.

### Types

```rust
pub struct PropertySpec {
    pub name: &'static str,
    pub syntax: &'static str,      // e.g. "<length> | <percentage> | auto"
    pub initial: &'static str,     // e.g. "auto", "0", "transparent"
    pub inherited: bool,
}

pub enum ValidationResult {
    Valid,
    Warn(String),    // suspicious but might be intentional
    Invalid(String), // clearly wrong
}
```

### API

```rust
/// Look up a CSS property by name.
pub fn property(name: &str) -> Option<&'static PropertySpec>

/// Validate a value against a property's expected types.
pub fn validate_value(property: &str, value: &str) -> ValidationResult
```

### Value validation coverage

- Lengths: `10px`, `2em`, `0`, `100vh`
- Colors: `#fff`, `#007acc`, `rgb()`, `hsl()`, named colors, `transparent`, `currentColor`
- Percentages: `50%`
- Keywords: property-specific (`auto`, `none`, `inherit`, `flex`, etc.)
- Pass-through (always valid): `var(--*)`, `calc()`, `env()`
- Shorthands: `border`, `background`, etc. get looser validation

### Offline builds

The build script caches fetched spec data in `data/` (checked into git). Builds work offline using the cache. Cache is refreshed explicitly.

## Crate: css-styled-derive

Proc macro crate providing `#[derive(StyledComponent)]` and the `css!()` macro.

### Derive: StyledComponent

```rust
#[derive(StyledComponent)]
#[component(scope = "split-handle")]
#[component(class(bar = "split-handle-bar"))]
#[component(modifier(horizontal, vertical))]
pub struct SplitHandleStyle {
    #[prop(css = "width", on = bar)]
    pub thickness: String,

    #[prop(css = "width")]
    pub hover_target_thickness: String,

    #[prop(css = "background", on = bar)]
    pub color: String,

    #[prop(css = "background", on = bar, pseudo = ":hover")]
    pub hover_color: String,
}
```

#### Struct-level attributes

| Attribute | Required | Description |
|---|---|---|
| `scope = "name"` | yes | Root CSS class name |
| `class(alias = "name", ...)` | no | Named child class selectors |
| `modifier(name, ...)` | no | Modifier class variants |

#### Field-level attributes

| Attribute | Required | Description |
|---|---|---|
| `css = "property"` | yes | CSS property name (compile-time validated) |
| `on = alias` | no | Target a named child class (default: scope element) |
| `pseudo = ":hover"` | no | Pseudo-class/element (compile-time validated) |
| `skip` | no | Exclude field from CSS generation |

Fields without `#[prop(...)]` or `#[css(skip)]` are a compile error.

#### Generated code

The derive generates:

1. Associated constants for all declared names:
   ```rust
   impl SplitHandleStyle {
       pub const SCOPE: &str = "split-handle";
       pub const BAR: &str = "split-handle-bar";
       pub const HORIZONTAL: &str = "horizontal";
       pub const VERTICAL: &str = "vertical";
   }
   ```

2. `impl IntoCss for SplitHandleStyle` that groups fields by (on, pseudo) into CSS rule blocks, scoped under the scope class.

3. `impl Default for SplitHandleStyle` is expected to be provided by the user.

#### Compile-time checks

- `css` value is a known CSS property (fuzzy-match suggestions on typo)
- `pseudo` is a known CSS pseudo-class/element (fuzzy-match suggestions on typo)
- `on` references a declared class alias (not a raw string)
- Duplicate property + selector + pseudo combinations are flagged

### Macro: css!

Validates a CSS block against the spec at compile time, resolving named selectors from a `StyledComponent` struct.

```rust
impl StyledComponentBase for SplitHandleStyle {
    fn base_css() -> &'static str {
        css!(SplitHandleStyle, {
            SCOPE {
                display: flex;
                align-items: center;
                flex-shrink: 0;
            }
            SCOPE.HORIZONTAL {
                cursor: col-resize;
            }
            SCOPE.HORIZONTAL BAR {
                height: 100%;
            }
            SCOPE.VERTICAL {
                cursor: row-resize;
            }
            SCOPE.VERTICAL BAR {
                width: 100%;
            }
            BAR {
                transition: background 0.1s ease;
                pointer-events: none;
            }
        })
    }
}
```

The macro:

- Resolves `SCOPE`, `BAR`, `HORIZONTAL`, `VERTICAL` to the struct's generated constants. Unknown names are a compile error.
- Validates every CSS property name against the spec.
- Validates every CSS value against the property's expected types.
- Outputs a `&'static str` of valid, scoped CSS.

## Crate: css-styled (runtime)

The public-facing crate that ties everything together.

### Traits

```rust
pub trait IntoCss {
    /// Returns the scoped CSS string for the dynamic (themed) properties.
    fn to_css(&self) -> String;

    /// Returns the scope class name.
    fn scope(&self) -> &'static str;
}

pub trait StyledComponentBase: IntoCss {
    /// Returns the static/structural CSS for this component.
    fn base_css() -> &'static str { "" }
}
```

`to_css()` prepends `base_css()` to the dynamic field output.

### Runtime value validation

When `to_css()` is called, each field value is validated via `css_spec_data::validate_value()`. Warnings are emitted via `web_sys::console::warn_1` (wasm) or `eprintln!` (native). CSS is still emitted regardless.

### Leptos feature

With `features = ["leptos"]`:

```rust
/// Renders a single <style> tag aggregating CSS from multiple styled components.
#[component]
pub fn StyleTag(styles: Vec<&dyn IntoCss>) -> impl IntoView {
    let css: String = styles.iter().map(|s| s.to_css()).collect::<Vec<_>>().join("\n");
    view! { <style>{css}</style> }
}
```

## Error messages

The derive macro provides helpful compile-time errors with suggestions:

```
error: unknown CSS property "widht"
  --> src/style.rs:5:18
   |
5  |     #[prop(css = "widht")]
   |                  ^^^^^^^
   = help: did you mean "width"?
```

```
error: unknown CSS pseudo-class ":hovr"
  --> src/style.rs:8:21
   |
8  |     #[prop(pseudo = ":hovr")]
   |                     ^^^^^^^
   = help: did you mean ":hover"?
```

```
error: unknown selector name "BARF"
  --> src/style.rs:12:5
   |
12 |     BARF { display: flex; }
   |     ^^^^
   = help: available names: SCOPE, BAR, HORIZONTAL, VERTICAL
```

```
error: field `border_color` has no #[prop(...)] attribute
  --> src/style.rs:10:5
   |
10 |     border_color: String,
   |     ^^^^^^^^^^^^
   = help: add #[prop(css = "...")] or #[prop(skip)] to this field
```

## End-to-end example: mullion split handle

### Style definition

```rust
// src/components/split_handle_style.rs

#[derive(StyledComponent, Clone, Debug)]
#[component(scope = "split-handle")]
#[component(class(bar = "split-handle-bar"))]
#[component(modifier(horizontal, vertical))]
pub struct SplitHandleStyle {
    #[prop(css = "width", on = bar)]
    pub thickness: String,

    #[prop(css = "width")]
    pub hover_target_thickness: String,

    #[prop(css = "background", on = bar)]
    pub color: String,

    #[prop(css = "background", on = bar, pseudo = ":hover")]
    pub hover_color: String,
}

impl Default for SplitHandleStyle {
    fn default() -> Self {
        Self {
            thickness: "4px".into(),
            hover_target_thickness: "8px".into(),
            color: "transparent".into(),
            hover_color: "#007acc".into(),
        }
    }
}

impl StyledComponentBase for SplitHandleStyle {
    fn base_css() -> &'static str {
        css!(SplitHandleStyle, {
            SCOPE {
                display: flex;
                align-items: center;
                flex-shrink: 0;
            }
            SCOPE.HORIZONTAL {
                cursor: col-resize;
            }
            SCOPE.HORIZONTAL BAR {
                height: 100%;
            }
            SCOPE.VERTICAL {
                cursor: row-resize;
            }
            SCOPE.VERTICAL BAR {
                width: 100%;
            }
            BAR {
                transition: background 0.1s ease;
                pointer-events: none;
            }
        })
    }
}
```

### Component

```rust
// src/components/split_handle.rs

#[component]
pub fn SplitHandle(
    direction: SplitDirection,
    on_resize: Callback<f64>,
    style: SplitHandleStyle,
) -> impl IntoView {
    let modifier = match direction {
        SplitDirection::Horizontal => SplitHandleStyle::HORIZONTAL,
        SplitDirection::Vertical => SplitHandleStyle::VERTICAL,
    };

    let on_mousedown = move |ev: MouseEvent| {
        // ... drag logic unchanged
    };

    view! {
        <div class=format!("{} {}", SplitHandleStyle::SCOPE, modifier) on:mousedown=on_mousedown>
            <div class=SplitHandleStyle::BAR />
        </div>
    }
}
```

### App root

```rust
// src/components/mullion_root.rs

#[component]
pub fn MullionRoot<D: PaneData + Send + Sync>(/* ... */) -> impl IntoView {
    let split_style = use_context::<SplitHandleStyle>().unwrap_or_default();
    let activity_style = use_context::<ActivityBarStyle>().unwrap_or_default();

    view! {
        <StyleTag styles=vec![&split_style, &activity_style] />
        <div>
            // ... pane tree
        </div>
    }
}
```

### Consumer configuration

```rust
// The app provides themed values via Leptos context
provide_context(SplitHandleStyle {
    thickness: "2px".into(),
    hover_target_thickness: "8px".into(),
    color: "#1a1a1a".into(),
    hover_color: "#333".into(),
});
```

## Generated CSS output

For the split handle with the consumer config above, `to_css()` produces:

```css
/* base (static) */
.split-handle { display: flex; align-items: center; flex-shrink: 0; }
.split-handle.horizontal { cursor: col-resize; }
.split-handle.horizontal .split-handle-bar { height: 100%; }
.split-handle.vertical { cursor: row-resize; }
.split-handle.vertical .split-handle-bar { width: 100%; }
.split-handle-bar { transition: background 0.1s ease; pointer-events: none; }

/* dynamic (from struct fields) */
.split-handle { width: 8px; }
.split-handle .split-handle-bar { width: 2px; background: #1a1a1a; }
.split-handle:hover .split-handle-bar { background: #333; }
```

## Testing strategy

### css-spec-data
- Unit tests: generated tables contain known properties (`width`, `background`, `display`, etc.)
- Unit tests: `validate_value` — good values pass, bad values return `Invalid`, `var(--foo)`/`calc()` pass through
- Snapshot test of a subset of generated data to catch regressions on spec updates

### css-styled-derive
- `trybuild` compile-pass/compile-fail tests:
  - Valid derive compiles
  - `#[prop(css = "widht")]` fails with helpful typo suggestion
  - `#[prop(pseudo = ":hovr")]` fails with suggestion
  - Missing `#[prop(...)]` on non-skipped field fails
  - Duplicate property+selector+pseudo fails
  - Unknown selector name in `css!` fails
- Output tests: macro-expand a struct, assert `to_css()` matches expected CSS

### css-styled
- Integration tests: derive struct, set values, call `to_css()`, assert output
- Runtime validation tests: bad value emits warning, CSS still produced
- Leptos feature tests: `StyleTag` renders correct `<style>` content

## Non-goals

- Full CSS preprocessor (no nesting, no variables, no mixins beyond what CSS natively supports)
- Runtime theme switching (consumers can use Leptos signals/context for that)
- CSS-in-JS style API (this is CSS-in-Rust with validation, not a new language)
