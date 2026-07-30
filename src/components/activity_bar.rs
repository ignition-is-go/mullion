use std::collections::HashSet;

use leptos::prelude::*;

use crate::activity::{ActivityIcon, ActivityNode};
use crate::context::MullionContext;
use crate::drag::DragPayload;
use crate::theme::MullionTheme;
use crate::tree::{ActivityId, CategoryId, PaneData, PaneId, SplitDirection};


/// Colour of the *active* floating activity's icon.
///
/// Floating activities are registered outside any category, so unlike categorised
/// ones they have no `Category::color` to borrow for their active state. Hosts can
/// colour-code them by defining `--ab-float-active-color`; otherwise this falls
/// back to the primary text colour.
///
/// Deliberately a foreground colour. This was `theme.accent`, which is a
/// *background* by contract — documented as "active tabs, hover backgrounds",
/// defaulted to `#222222`, and used elsewhere only as `--ws-btn-bg` — so the
/// active floating icon rendered invisible against the bar.
const FLOAT_ACTIVE_COLOR: &str = "var(--ab-float-active-color, var(--ml-text))";

/// Internal CSS variables for the activity bar — not exposed to consumers.
#[derive(css_styled::CssVars)]
struct ActivityBarInternal {
    #[var("--ab-cat-color")]
    pub category_color: String,
}

/// Behavior options for the activity bar. Provide via Leptos context before
/// mounting `MullionProvider` or `MullionRoot` to override defaults.
///
/// Unlike `ActivityBarStyle` (which controls appearance via CSS variables),
/// this struct controls interaction semantics that can't be expressed as a
/// single CSS property.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActivityBarBehavior {
    /// When `true` (the default), the activity bar widens on hover to reveal
    /// activity labels. Set to `false` to pin it at its collapsed width —
    /// useful when labels would overflow surrounding UI, or when the host
    /// app wants a purely icon-driven bar.
    ///
    /// Auto-hiding a bar off the pane edge is a *per-pane* decision, not a
    /// global one — see [`crate::context::PaneAutoHideActivityBar`].
    pub hover_expand: bool,
}

impl Default for ActivityBarBehavior {
    fn default() -> Self {
        Self { hover_expand: true }
    }
}

/// Style for the activity bar, powered by css-styled.
///
/// All customizable values are CSS custom properties. Hover behavior and
/// structural layout come from base CSS. Active/inactive opacity is applied
/// via inline styles since it varies per-button at runtime.
#[derive(css_styled::StyledComponent, Clone, Debug)]
#[component(scope = "mullion-ab")]
#[component(theme = MullionTheme)]
#[component(class(
    panel = "mullion-ab-panel",
    label = "mullion-ab-label",
    icon_slot = "mullion-ab-icon-slot",
    btn = "mullion-ab-btn",
    dot = "mullion-ab-dot",
    cat_border = "mullion-ab-cat-border",
    icon = "mullion-ab-icon"
))]
#[component(modifier(collapsed, auto_hide, dragging))]
#[component(internals(ActivityBarInternal))]
#[component(base_css)]
pub struct ActivityBarStyle {
    #[prop(var = "--ab-width", default = "28px")]
    pub width: String,
    #[prop(var = "--ab-expanded-width", default = "150px")]
    pub expanded_width: String,
    #[prop(var = "--ab-icon-size", default = "14px")]
    pub icon_size: String,
    #[prop(var = "--ab-background", default = theme.surface)]
    pub background: String,
    #[prop(var = "--ab-border", default = "1px solid var(--ml-border)")]
    pub border: String,
    #[prop(var = "--ab-border-radius", default = "0")]
    pub border_radius: String,
    #[prop(var = "--ab-expanded-padding", default = "8px")]
    pub expanded_padding: String,
    #[prop(var = "--ab-font-size", default = "11px")]
    pub font_size: String,
    #[prop(var = "--ab-icon-color", default = theme.text)]
    pub icon_color: String,
    #[prop(var = "--ab-icon-stroke-color", default = theme.text)]
    pub icon_stroke_color: String,
    #[prop(var = "--ab-icon-opacity", default = "0.5")]
    pub icon_opacity: String,
    #[prop(var = "--ab-icon-active-opacity", default = "1")]
    pub icon_active_opacity: String,
    #[prop(var = "--ab-cat-border-width", default = "2px")]
    pub category_border_width: String,
    /// Background of an *open* category's card.
    ///
    /// Must be translucent: nested categories render nested cards, so the alpha
    /// composites and each level reads a step stronger than its parent without
    /// needing per-depth colours. A solid colour would flatten that to one tone.
    /// The default suits mullion's dark defaults; light-theme hosts should
    /// override with a dark translucent instead.
    #[prop(var = "--ab-cat-card-bg", default = "rgba(255,255,255,0.045)")]
    pub category_card_background: String,
    /// Hairline at the top of an *open* category's card, marking where the group
    /// starts. Translucent for the same reason as the card fill.
    #[prop(var = "--ab-cat-edge", default = "rgba(255,255,255,0.08)")]
    pub category_edge: String,
    /// Label colour on category rows. Muted by default, so a category reads as a
    /// grouping rather than a destination.
    #[prop(var = "--ab-cat-label-color", default = theme.text_muted)]
    pub category_label_color: String,
}

impl css_styled::StyledComponentBase for ActivityBarStyle {
    fn base_css() -> &'static str {
        css_styled::css!(ActivityBarStyle, {
            SCOPE {
                flex-shrink: 0;
                position: relative;
                width: var(--ab-width);
            }
            PANEL {
                position: absolute;
                top: 0;
                left: 0;
                bottom: 0;
                background: var(--ab-background);
                border-right: var(--ab-border);
                border-radius: var(--ab-border-radius);
                z-index: 10;
                display: flex;
                flex-direction: column;
                justify-content: space-between;
                overflow-y: auto;
                overflow-x: hidden;
                scrollbar-width: none;
                width: var(--ab-width);
                padding-right: 0;
                transition: width 0.15s ease, padding-right 0.15s ease, transform 0.15s ease;
            }
            /* Hold the bar open for the duration of a drag. Chrome drops
               `:hover` as soon as a native drag starts, so without this the
               panel collapses mid-gesture and the row being dragged shrinks out
               from under the cursor. That is not what was cancelling drags (see
               the `request_animation_frame` note on the drag source for the real
               cause) — but a drag source that resizes underneath the pointer is a
               hazard worth removing, and holding the bar open also keeps the drag
               image stable. */
            SCOPE.DRAGGING PANEL {
                width: var(--ab-expanded-width);
                transition: none;
            }
            SCOPE.DRAGGING LABEL {
                opacity: 1;
                transition: none;
            }
            SCOPE:not(.COLLAPSED):hover PANEL {
                width: var(--ab-expanded-width);
                padding-right: var(--ab-expanded-padding);
            }
            /* Auto-hide: the bar reserves no space and its panel sits fully off the
               left edge (clipped by the pane's overflow:hidden, so it never bleeds
               into a neighbouring pane). An invisible edge strip (::before) is the
               hover target that summons it — hovering the strip slides the panel in
               over the pane content; it stays open while the cursor is over the
               revealed panel, and slides back out on leave. */
            SCOPE.AUTO_HIDE {
                width: 0;
            }
            SCOPE.AUTO_HIDE::before {
                content: "";
                position: absolute;
                left: 0;
                top: 0;
                bottom: 0;
                width: 12px;
                z-index: 9;
            }
            SCOPE.AUTO_HIDE PANEL {
                transform: translateX(-100%);
            }
            SCOPE.AUTO_HIDE:hover PANEL {
                transform: translateX(0);
            }
            LABEL {
                /* Hidden by zero width + opacity rather than `display:none`, so
                   the label never leaves the box tree while a drag is in flight.
                   `flex-shrink` takes it to 0px against the fixed-width icon slot
                   when the bar is collapsed and `overflow:hidden` clips the text,
                   so it is invisible either way. Not a drag source — the row is,
                   because the row is the element that is always rendered. */
                opacity: 0;
                min-width: 0;
                overflow: hidden;
                text-overflow: ellipsis;
                transition: opacity 0.15s ease;
            }
            SCOPE:not(.COLLAPSED):hover LABEL {
                opacity: 1;
            }
            ICON_SLOT {
                width: var(--ab-width);
                flex-shrink: 0;
                display: flex;
                align-items: center;
                justify-content: center;
            }
            BTN {
                display: flex;
                align-items: center;
                height: var(--ab-width);
                cursor: pointer;
                /* Without this, mousedown on a label's text starts a text
                   selection, which pre-empts the HTML5 drag — so an activity
                   would only be draggable by its icon, never by its name.
                   Toolbar chrome shouldn't be selectable anyway. */
                user-select: none;
                /* `draggable="true"` alone is not enough for a text-bearing
                   element in Chrome: the text still wins the mousedown, so an
                   activity could be dragged by its icon but never by its name.
                   `-webkit-user-drag: element` makes the whole element the drag
                   source explicitly. Non-standard but supported in Chrome and
                   Safari; harmless elsewhere. */
                -webkit-user-drag: element;
                white-space: nowrap;
                border: none;
                background: none;
                width: 100%;
                text-align: left;
                font-size: var(--ab-font-size);
                padding: 0;
                color: var(--ab-icon-color);
                opacity: var(--ab-icon-opacity);
                position: relative;
            }
            ICON {
                display: flex;
                align-items: center;
                justify-content: center;
                width: var(--ab-icon-size);
                height: var(--ab-icon-size);
                flex-shrink: 0;
                overflow: hidden;
                stroke: var(--ab-icon-stroke-color);
            }
            DOT {
                position: absolute;
                left: 2px;
                top: 50%;
                transform: translateY(-50%);
                width: 4px;
                height: 4px;
                border-radius: 50%;
                background: var(--ab-cat-color);
            }
            CAT_BORDER {
                position: absolute;
                left: 0;
                top: 0;
                bottom: 0;
                width: var(--ab-cat-border-width);
                background: var(--ab-cat-color);
            }
        })
    }
}

/// Renders the activity bar for a single pane.
///
/// Shows categories as clickable icons. On hover (pure CSS), expands to show
/// activity names. Clicking a category toggles its expanded activity list.
#[component]
pub fn ActivityBar<D: PaneData + Send + Sync>(
    pane_id: PaneId,
    data: Signal<D>,
    ctx: MullionContext<D>,
    #[prop(optional)] app_icon: Option<ActivityIcon>,
    /// When `true`, this pane's bar tucks off the left edge and reveals on
    /// edge-hover (resolved per-pane by the host). Default: pinned/visible.
    #[prop(default = false)]
    auto_hide: bool,
) -> impl IntoView {
    let style = ctx.activity_bar_style.clone();

    // Which categories are open. A set rather than a single id because the tree
    // nests: opening a child must not close its parent, or the child would
    // vanish along with it.
    let expanded = RwSignal::new(HashSet::<CategoryId>::new());

    // This pane's slice of the registered tree, filtered by its data.
    let ctx_for_items = ctx.clone();
    let bar_items = Memo::new(move |_| {
        let d = data.get();
        ctx_for_items
            .items
            .with_value(|items| filter_nodes(items, &d))
    });

    let ctx_for_active = ctx.clone();
    let pid_for_active = pane_id.clone();
    let active_activity = Memo::new(move |_| {
        ctx_for_active
            .tree
            .with(|tree| match tree.find(&pid_for_active) {
                Some(crate::tree::PaneNode::Leaf {
                    active_activity, ..
                }) => active_activity.clone(),
                _ => None,
            })
    });

    // Reveal the active activity by opening its whole ancestor chain — opening
    // only the nearest category would leave it hidden inside collapsed
    // grandparents.
    let ctx_for_expand = ctx.clone();
    Effect::new(move |_| {
        if let Some(act_id) = active_activity.get() {
            let ancestors = ctx_for_expand.activity_ancestors(&act_id);
            if !ancestors.is_empty() {
                expanded.update(|open| open.extend(ancestors));
            }
        }
    });

    let renderer = BarRender {
        ctx: ctx.clone(),
        pane_id: pane_id.clone(),
        expanded,
        active_opacity: style.icon_active_opacity.clone(),
    };

    let ctx_actions = ctx.clone();

    // Host-provided per-pane chrome (e.g. session indicator). Cloned out before
    // `ctx` is moved into the item closure below.
    let pane_accessory = ctx.pane_accessory.clone();
    let pid_accessory = pane_id.clone();

    // Reactive on `drag`: while a drag is in flight the bar gets the `dragging`
    // modifier so it stays open (see the CSS for why that is load-bearing, not
    // cosmetic). This is a class swap on an existing element, not a re-render —
    // replacing the node mid-drag would cancel the drag just as surely.
    let hover_expand = ctx.activity_bar_behavior.hover_expand;
    let ctx_drag_class = ctx.clone();
    let scope_class = move || {
        let mut mods = Vec::new();
        if !hover_expand {
            mods.push(ActivityBarModifier::Collapsed);
        }
        if auto_hide {
            mods.push(ActivityBarModifier::AutoHide);
        }
        if ctx_drag_class.drag.get().is_some() {
            mods.push(ActivityBarModifier::Dragging);
        }
        if mods.is_empty() {
            ActivityBarStyle::SCOPE.to_string()
        } else {
            ActivityBarStyle::class(&mods)
        }
    };

    view! {
        <div class=scope_class>
            <div class=ActivityBarStyle::PANEL>
                // App icon + the registered item tree
                <div>
                    {app_icon.map(|icon| {
                        let ctx_drag = ctx.clone();
                        let ctx_dragend = ctx.clone();
                        let pid_drag = pane_id.clone();
                        view! {
                            <div class=ActivityBarStyle::BTN
                                 style="cursor:grab"
                                 draggable="true"
                                 on:dragstart=move |ev| {
                                     ctx_drag.drag.set(Some(DragPayload::Pane(pid_drag.clone())));
                                     if let Some(dt) = ev.data_transfer() {
                                         let _ = dt.set_data("text/plain", &pid_drag.0);
                                         dt.set_effect_allowed("move");
                                     }
                                 }
                                 on:dragend=move |_| {
                                     ctx_dragend.drag.set(None);
                                 }>
                                <span class=ActivityBarStyle::ICON_SLOT>
                                    {render_icon(&icon)}
                                </span>
                                <span class=ActivityBarStyle::LABEL style="font-weight:600;font-size:12px"></span>
                            </div>
                        }
                    })}
                    {move || {
                        let active = active_activity.get();
                        renderer.nodes(&bar_items.get(), 0, None, active.as_ref())
                    }}
                </div>
                // Pane actions (bottom)
                <div>
                    // Host-provided per-pane chrome (session indicator, etc.),
                    // anchored above the split/close controls.
                    {pane_accessory.map(move |f| f(pid_accessory.clone()))}
                    {
                        let ctx_sh = ctx_actions.clone();
                        let ctx_sv = ctx_actions.clone();
                        let ctx_cl = ctx_actions.clone();
                        let pid_sh = pane_id.clone();
                        let pid_sv = pane_id.clone();
                        let pid_cl = pane_id.clone();
                        view! {
                            <button class=ActivityBarStyle::BTN on:click=move |_| {
                                let d = data.get();
                                let new_id = PaneId::new(format!("{:.0}", web_sys::js_sys::Math::random() * 1e12));
                                ctx_sh.split_pane(&pid_sh, SplitDirection::Horizontal, new_id, d);
                            }>
                                <span class=ActivityBarStyle::ICON_SLOT><span class=ActivityBarStyle::ICON inner_html=ICON_SPLIT_H></span></span>
                                <span class=ActivityBarStyle::LABEL>"Split H"</span>
                            </button>
                            <button class=ActivityBarStyle::BTN on:click=move |_| {
                                let d = data.get();
                                let new_id = PaneId::new(format!("{:.0}", web_sys::js_sys::Math::random() * 1e12));
                                ctx_sv.split_pane(&pid_sv, SplitDirection::Vertical, new_id, d);
                            }>
                                <span class=ActivityBarStyle::ICON_SLOT><span class=ActivityBarStyle::ICON inner_html=ICON_SPLIT_V></span></span>
                                <span class=ActivityBarStyle::LABEL>"Split V"</span>
                            </button>
                            <button class=ActivityBarStyle::BTN on:click=move |_| { ctx_cl.close_pane(&pid_cl); }>
                                <span class=ActivityBarStyle::ICON_SLOT><span class=ActivityBarStyle::ICON inner_html=ICON_CLOSE></span></span>
                                <span class=ActivityBarStyle::LABEL>"Close"</span>
                            </button>
                        }
                    }
                </div>
            </div>
        </div>
    }
}

/// A pane-filtered projection of the registered item tree.
///
/// Projected rather than rendered straight from [`ActivityNode`] so it can be
/// `PartialEq` and therefore live in a `Memo`: `ActivityDef` holds `fn` pointers
/// and isn't comparable, and without memo dedup the bar would rebuild on every
/// unrelated tree change — churning the DOM, and mid-drag destroying the drag
/// source.
#[derive(Clone, PartialEq)]
enum BarNode {
    Activity {
        id: ActivityId,
        name: String,
        icon: ActivityIcon,
    },
    Category {
        id: CategoryId,
        name: String,
        icon: ActivityIcon,
        color: String,
        children: Vec<BarNode>,
    },
}

impl BarNode {
    /// Whether `active` is this node or anywhere beneath it. Drives the collapsed
    /// category's dot indicator, which has to survive nesting: an active
    /// grandchild must still mark its top-level ancestor.
    fn contains_active(&self, active: Option<&ActivityId>) -> bool {
        match self {
            BarNode::Activity { id, .. } => active == Some(id),
            BarNode::Category { children, .. } => {
                children.iter().any(|c| c.contains_active(active))
            }
        }
    }
}

/// Project the registered tree down to what one pane shows.
///
/// An activity survives if its `filter` passes; a category survives only if some
/// descendant activity does, so a category whose entire contents are filtered out
/// disappears instead of expanding to nothing.
fn filter_nodes<D: PaneData>(items: &[ActivityNode<D>], data: &D) -> Vec<BarNode> {
    items
        .iter()
        .filter_map(|node| match node {
            ActivityNode::Activity(def) => (def.filter)(data).then(|| BarNode::Activity {
                id: def.id.clone(),
                name: def.name.clone(),
                icon: def.icon.clone(),
            }),
            ActivityNode::Category(cat) => {
                let children = filter_nodes(&cat.children, data);
                (!children.is_empty()).then(|| BarNode::Category {
                    id: cat.id.clone(),
                    name: cat.name.clone(),
                    icon: cat.icon.clone(),
                    color: cat.color.clone(),
                    children,
                })
            }
        })
        .collect()
}

/// Renders the item tree for one pane's bar.
///
/// A struct rather than a pile of arguments threaded through the recursion, and
/// it keeps the activity row in exactly one place — the previous flat/categorised
/// split meant every fix to a row had to be made twice.
struct BarRender<D: PaneData> {
    ctx: MullionContext<D>,
    pane_id: PaneId,
    expanded: RwSignal<HashSet<CategoryId>>,
    active_opacity: String,
}

impl<D: PaneData + Send + Sync> BarRender<D> {
    /// Render a sibling list. `inherited` is the nearest enclosing category's
    /// colour, which activities use for their active state; `None` at the top
    /// level, where there is no category to borrow from.
    fn nodes(
        &self,
        nodes: &[BarNode],
        depth: usize,
        inherited: Option<&str>,
        active: Option<&ActivityId>,
    ) -> Vec<AnyView> {
        nodes
            .iter()
            .map(|node| match node {
                BarNode::Activity { id, name, icon } => {
                    self.activity(id, name, icon, inherited, active)
                }
                BarNode::Category { .. } => self.category(node, depth, active),
            })
            .collect()
    }

    /// One activity row: selects on click, and is a drag source when the host
    /// installed a `new_pane` hook.
    fn activity(
        &self,
        id: &ActivityId,
        name: &str,
        icon: &ActivityIcon,
        inherited: Option<&str>,
        active: Option<&ActivityId>,
    ) -> AnyView {
        let is_active = active == Some(id);
        // A categorised activity highlights in its category's colour; a
        // top-level one has no category, so it falls back to the themeable
        // foreground (never `theme.accent`, which is a background).
        let active_color = inherited.unwrap_or(FLOAT_ACTIVE_COLOR);
        let active_style = if is_active {
            ActivityBarStyle::vars(|v| {
                v.icon_opacity(&self.active_opacity)
                    .icon_color(active_color)
                    .icon_stroke_color(active_color)
            })
        } else {
            String::new()
        };

        let can_drag = self.ctx.new_pane.is_some();
        let (ctx_click, ctx_ds, ctx_de, ctx_key) = (
            self.ctx.clone(),
            self.ctx.clone(),
            self.ctx.clone(),
            self.ctx.clone(),
        );
        let (pid_click, pid_key) = (self.pane_id.clone(), self.pane_id.clone());
        let (act_click, act_drag, act_key) = (id.clone(), id.clone(), id.clone());
        let label = name.to_string();
        let icon_view = render_icon(icon);

        // A `div role=button`, not a `<button>`: form controls consume mousedown
        // for activation, so browsers won't reliably start an HTML5 drag from one
        // (Firefox ignores `draggable` on them outright). Keyboard activation is
        // restored with tabindex + Enter/Space.
        view! {
            <div class=ActivityBarStyle::BTN
                 role="button"
                 tabindex="0"
                 style=active_style
                 draggable=can_drag.then_some("true")
                 on:dragstart=move |ev| {
                     if !can_drag { return }
                     if let Some(dt) = ev.data_transfer() {
                         let _ = dt.set_data("text/plain", &act_drag.0);
                         dt.set_effect_allowed("copy");
                     }
                     // Deferred out of the dragstart handler on purpose. Setting
                     // this mounts every pane's DropOverlay, and an element
                     // appearing under the pointer while the drag session is
                     // still being established makes Chrome abandon the drag:
                     // dragstart fires, then dragend with dropEffect=none and no
                     // dragover anywhere, not even a document-level capture
                     // listener. Dragging by the icon escaped it only because the
                     // cursor sits left of the content area, where nothing is
                     // inserted.
                     let ctx_deferred = ctx_ds.clone();
                     let payload = DragPayload::NewActivity(act_drag.clone());
                     request_animation_frame(move || {
                         ctx_deferred.drag.set(Some(payload));
                     });
                 }
                 on:dragend=move |_| ctx_de.drag.set(None)
                 on:keydown=move |ev: web_sys::KeyboardEvent| {
                     if ev.key() == "Enter" || ev.key() == " " {
                         ev.prevent_default();
                         ctx_key.set_active_activity(&pid_key, Some(act_key.clone()));
                     }
                 }
                 on:click=move |_| {
                     ctx_click.set_active_activity(&pid_click, Some(act_click.clone()));
                 }>
                <span class=ActivityBarStyle::ICON_SLOT>{icon_view}</span>
                <span class=ActivityBarStyle::LABEL>{label}</span>
            </div>
        }
        .into_any()
    }

    /// One category row plus, when open, its children indented one level.
    fn category(&self, node: &BarNode, depth: usize, active: Option<&ActivityId>) -> AnyView {
        let BarNode::Category {
            id,
            name,
            icon,
            color,
            children,
        } = node
        else {
            return ().into_any();
        };

        let expanded = self.expanded;
        let is_open = expanded.get().contains(id);
        let has_active = node.contains_active(active);
        let cat_style = if is_open || has_active {
            ActivityBarStyle::vars(|v| v.icon_opacity(&self.active_opacity))
        } else {
            String::new()
        };
        // Collapsed but holding the active activity — mark it, since the
        // highlighted row itself is hidden.
        let show_dot = !is_open && has_active;

        let id_toggle = id.clone();
        let label = name.to_string();
        let icon_view = render_icon(icon);
        let dot_color = color.clone();
        let border_color = color.clone();
        // One glyph, rotated — never two different glyphs. `▸` and `▾` have
        // different advance widths, so swapping them moved the chevron (and
        // everything else in the row) as a category opened. A transform has no
        // layout effect, and the fixed-width, centred box below keeps the slot
        // identical in both states.
        let chevron_rotation = if is_open {
            "transform:rotate(90deg);"
        } else {
            ""
        };

        let children_view = if is_open {
            let rendered = self.nodes(children, depth + 1, Some(color), active);
            Some(view! {
                <div style="position:relative">
                    <div class=ActivityBarStyle::CAT_BORDER
                         style=ActivityBarInternal::vars(|v| v.category_color(&border_color))></div>
                    {rendered}
                </div>
            })
        } else {
            None
        };

        // Only an *open* category gets the card. Closed, it is a plain row, so a
        // collapsed bar stays quiet; open, the card encloses the header and the
        // children together and its translucent fill composites with any parent
        // card, which is what makes nesting depth legible.
        //
        // Inline rather than a class: adding entries to this component's
        // `class(..)` list makes css-styled silently mis-resolve the *existing*
        // ones (BTN and friends came out as `.mullion-ab-panel0`, so every button
        // lost its base styling and fell back to browser chrome). The values are
        // still themeable through the two custom properties.
        // Geometry is identical open or closed — only colours change. A border
        // that appears on open would push every row below it down 1px, and a
        // negative margin that appears on open would shift this row's content
        // right, so both are always present and merely transparent when closed.
        //
        // The bleed pulls the card through the panel's right-hand padding so it
        // meets the sidebar edge rather than stopping short. Top level only:
        // a nested card's parent has already bled, so it spans to the edge for
        // free, and repeating it per level would push each one further out. No
        // radius anywhere — rounded corners read as a gap from the edge.
        let fill = if is_open {
            "var(--ab-cat-card-bg)"
        } else {
            "transparent"
        };
        let edge = if is_open {
            "var(--ab-cat-edge)"
        } else {
            "transparent"
        };
        let bleed = if depth == 0 {
            "margin-right:calc(-1 * var(--ab-expanded-padding));"
        } else {
            ""
        };
        let wrapper_style = format!("background:{fill};border-top:1px solid {edge};{bleed}");

        view! {
            <div style=wrapper_style>
                <button class=ActivityBarStyle::BTN
                        style=format!("{cat_style};font-weight:600")
                        on:click=move |_| {
                            expanded.update(|open| {
                                if !open.remove(&id_toggle) {
                                    open.insert(id_toggle.clone());
                                }
                            });
                        }>
                    <span class=ActivityBarStyle::ICON_SLOT>
                        {show_dot.then(|| view! {
                            <span class=ActivityBarStyle::DOT
                                  style=ActivityBarInternal::vars(|v| v.category_color(&dot_color))></span>
                        })}
                        {icon_view}
                    </span>
                    <span class=ActivityBarStyle::LABEL
                          style="color:var(--ab-cat-label-color)">{label}</span>
                    // Reuses LABEL so it appears and hides exactly with the
                    // label, which is also the only time there is room for it.
                    // Fixed width + centred so the glyph's own metrics can never
                    // move the row; open/closed differ only by the rotation.
                    <span class=ActivityBarStyle::LABEL
                          style=format!(
                              "margin-left:auto;width:14px;flex-shrink:0;\
                               font-size:9px;line-height:1;text-align:center;\
                               opacity:0.5;{chevron_rotation}")>
                        "\u{25b8}"
                    </span>
                </button>
                {children_view}
            </div>
        }
        .into_any()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use css_styled::IntoCss;

    /// Every class token in a stylesheet, without the leading dot.
    ///
    /// Hand-rolled rather than pulling in a regex dev-dependency for one test.
    fn class_names(css: &str) -> Vec<String> {
        let mut out = Vec::new();
        let bytes: Vec<char> = css.chars().collect();
        let mut i = 0;
        while i < bytes.len() {
            // A class selector, not a decimal like `.15s` in `transition` — CSS
            // identifiers cannot begin with a digit.
            let starts_ident = i + 1 < bytes.len()
                && (bytes[i + 1].is_ascii_alphabetic() || bytes[i + 1] == '_' || bytes[i + 1] == '-');
            if bytes[i] == '.' && starts_ident {
                let start = i + 1;
                let mut end = start;
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric() || bytes[end] == '-' || bytes[end] == '_')
                {
                    end += 1;
                }
                if end > start {
                    out.push(bytes[start..end].iter().collect());
                }
                i = end;
            } else {
                i += 1;
            }
        }
        out
    }


    #[test]
    fn collapsed_modifier_gates_hover_rules() {
        let css = ActivityBarStyle::default().to_css();
        // The :not(.collapsed) guard must appear on both hover rules so the
        // bar stays at its collapsed width when the modifier is applied.
        assert!(
            css.contains(":not(.collapsed):hover"),
            "expected :not(.collapsed):hover in base CSS, got: {css}"
        );
        // And a plain SCOPE:hover rule must not slip through — otherwise the
        // modifier wouldn't actually suppress the hover behavior.
        assert!(
            !css.contains(".mullion-ab:hover"),
            "unguarded .mullion-ab:hover rule present in base CSS: {css}"
        );
    }


    #[test]
    fn category_card_colour_is_translucent() {
        // Load-bearing, not cosmetic: nested categories render nested cards, and
        // depth is legible only because the fills composite. A solid default
        // would flatten every level to one tone.
        let css = ActivityBarStyle::default().to_css();
        let has_alpha = css.contains("rgba(") || css.contains("hsla(") || css.contains("/ 0.");
        assert!(
            has_alpha,
            "category card background must be translucent so nesting stacks: {css}"
        );
    }

    #[test]
    fn every_class_identifier_resolves() {
        // css-styled substitutes each distinct SCREAMING_CASE name via a
        // placeholder `css-s-{index}`, then replaces the one occurring earliest,
        // comparing with a strict `pos < earliest`. `css-s-1` is a *prefix* of
        // `css-s-10`: both match at the same offset, index 1 is visited first,
        // so index 1 wins and consumes only its own 7 characters — leaving the
        // trailing digit in the output as a literal.
        //
        // So the eleventh distinct name emits as `<name-at-index-1's-class>0`,
        // the twelfth as `...1`, and so on. Here index 1 is PANEL, which is why
        // the breakage showed up as `.mullion-ab-panel0`. It cost a debugging
        // round: it took out BTN's rule, buttons fell back to browser chrome, and
        // the symptoms read as a styling disaster rather than a name collision.
        //
        // Guard the shape of the failure, not a count and not one victim: a
        // mis-substituted name always leaves a trailing digit welded onto another
        // class, so NO class in the generated CSS may end in a digit. That catches
        // the whole family (`…panel0`, `…panel1`, …) whichever index collides, and
        // fires the moment someone adds an eleventh name — rather than when the
        // stripe visibly disappears, which is how this was found the first time.
        // Harmless once css-styled is fixed: it simply never fires.
        let css = ActivityBarStyle::default().to_css();
        for class in class_names(&css) {
            assert!(
                !class.ends_with(|c: char| c.is_ascii_digit()),
                "class `{class}` ends in a digit — a SCREAMING_CASE name was \
                 mis-substituted, which means this block has passed ten distinct \
                 names (modifiers count too):\n{css}"
            );
        }
        // And the classes the render actually attaches must be present.
        for expected in [
            ".mullion-ab-btn",
            ".mullion-ab-label",
            ".mullion-ab-icon-slot",
            ".mullion-ab-dot",
            ".mullion-ab-icon",
            ".mullion-ab-panel {",
            // The eleventh name in this block, and the one the collision used to
            // eat. Its presence proves the resolved css-styled rev actually
            // contains the padding fix — an unpinned git dependency cannot tell
            // you that on its own.
            ".mullion-ab-cat-border",
        ] {
            assert!(css.contains(expected), "missing {expected} in base CSS: {css}");
        }
    }

    #[test]
    fn filter_drops_categories_left_empty() {
        use crate::activity::{ActivityDef, Category};
        use serde::{Deserialize, Serialize};

        #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
        struct D {
            show: bool,
        }

        fn gated(id: &str) -> ActivityDef<D> {
            ActivityDef::new(
                ActivityId::new(id),
                id,
                ActivityIcon::Class(String::new()),
                |d: &D| d.show,
                |_, _| ().into_any(),
            )
        }

        let items = vec![ActivityNode::Category(Category {
            id: CategoryId::new("outer"),
            name: "Outer".into(),
            icon: ActivityIcon::Class(String::new()),
            color: "#fff".into(),
            children: vec![ActivityNode::Category(Category {
                id: CategoryId::new("inner"),
                name: "Inner".into(),
                icon: ActivityIcon::Class(String::new()),
                color: "#000".into(),
                children: vec![ActivityNode::activity(gated("hidden"))],
            })],
        })];

        // The only activity passes: both wrappers survive.
        assert_eq!(filter_nodes(&items, &D { show: true }).len(), 1);

        // It is filtered out: the empty inner category must not survive, and
        // neither must the outer one — otherwise the bar shows a category that
        // expands to nothing.
        assert!(
            filter_nodes(&items, &D { show: false }).is_empty(),
            "categories whose whole subtree is filtered out must disappear"
        );
    }

    #[test]
    fn contains_active_sees_through_nesting() {
        let leaf = BarNode::Activity {
            id: ActivityId::new("deep"),
            name: "Deep".into(),
            icon: ActivityIcon::Class(String::new()),
        };
        let nested = BarNode::Category {
            id: CategoryId::new("outer"),
            name: "Outer".into(),
            icon: ActivityIcon::Class(String::new()),
            color: "#fff".into(),
            children: vec![BarNode::Category {
                id: CategoryId::new("inner"),
                name: "Inner".into(),
                icon: ActivityIcon::Class(String::new()),
                color: "#000".into(),
                children: vec![leaf],
            }],
        };
        // A collapsed top-level category must still show its dot when the active
        // activity is a grandchild.
        assert!(nested.contains_active(Some(&ActivityId::new("deep"))));
        assert!(!nested.contains_active(Some(&ActivityId::new("elsewhere"))));
        assert!(!nested.contains_active(None));
    }

    #[test]
    fn floating_active_colour_is_a_foreground() {
        // Regression: this was `theme.accent`, a background colour by contract
        // (default #222222, used elsewhere as `--ws-btn-bg`), which made the
        // active floating activity's icon invisible against the bar.
        assert!(
            !FLOAT_ACTIVE_COLOR.contains("--ml-accent"),
            "floating active colour must not use the accent (a background): {FLOAT_ACTIVE_COLOR}"
        );
        assert!(
            FLOAT_ACTIVE_COLOR.contains("--ml-text"),
            "expected a text-colour fallback, got: {FLOAT_ACTIVE_COLOR}"
        );
        // Host override hook, and it must not reference the property it is
        // assigned to (`--ab-icon-color`), which would be a cyclic reference.
        assert!(FLOAT_ACTIVE_COLOR.contains("--ab-float-active-color"));
        assert!(
            !FLOAT_ACTIVE_COLOR.contains("--ab-icon-color"),
            "cyclic custom-property reference: {FLOAT_ACTIVE_COLOR}"
        );
    }

    #[test]
    fn dragging_modifier_holds_the_bar_open() {
        let css = ActivityBarStyle::default().to_css();
        // The bar must stay expanded while a drag is in flight — Chrome drops
        // `:hover` when a native drag starts, so the hover rule alone would let
        // the panel collapse and resize the drag source under the cursor.
        assert!(
            css.contains(".dragging"),
            "expected a .dragging rule in base CSS, got: {css}"
        );
        // And it must not animate: a transitioning collapse is the same hazard,
        // just spread over 150ms.
        let dragging_block = css
            .split(".dragging")
            .nth(1)
            .expect("`.dragging` rule present");
        assert!(
            dragging_block.contains("transition:none") || dragging_block.contains("transition: none"),
            "expected transition:none in the .dragging rule, got: {dragging_block}"
        );
    }

    #[test]
    fn dragging_class_has_expected_name() {
        assert_eq!(
            ActivityBarStyle::class(&[ActivityBarModifier::Dragging]),
            "mullion-ab dragging",
        );
    }

    #[test]
    fn collapsed_class_has_expected_name() {
        assert_eq!(
            ActivityBarStyle::class(&[ActivityBarModifier::Collapsed]),
            "mullion-ab collapsed",
        );
    }

    #[test]
    fn behavior_defaults_to_hover_expand_true() {
        assert!(ActivityBarBehavior::default().hover_expand);
    }
}

fn render_icon(icon: &ActivityIcon) -> AnyView {
    let icon_class = ActivityBarStyle::ICON;
    match icon {
        ActivityIcon::Class(class) => {
            view! { <span class=format!("{} {}", icon_class, class)></span> }.into_any()
        }
        ActivityIcon::Svg(svg) => {
            let normalized = normalize_svg(svg);
            view! { <span class=icon_class inner_html={normalized}></span> }.into_any()
        }
        ActivityIcon::Url(url) => {
            view! { <img class=icon_class src={url.clone()} style="object-fit:contain" /> }
                .into_any()
        }
    }
}

fn normalize_svg(svg: &str) -> String {
    let mut result = svg.to_string();
    if let Some(pos) = result.find("<svg") {
        let insert_at = pos + 4;
        result.insert_str(insert_at, " style=\"width:100%;height:100%;display:block\"");
    }
    result
}

const ICON_SPLIT_H: &str = r#"<svg viewBox="0 0 16 16" fill="currentColor"><path d="M14 1H2a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V2a1 1 0 0 0-1-1zM2 2h5.5v12H2V2zm6.5 12V2H14v12H8.5z"/></svg>"#;

const ICON_SPLIT_V: &str = r#"<svg viewBox="0 0 16 16" fill="currentColor"><path d="M14 1H2a1 1 0 0 0-1 1v12a1 1 0 0 0 1 1h12a1 1 0 0 0 1-1V2a1 1 0 0 0-1-1zM2 2h12v5.5H2V2zm0 6.5h12V14H2V8.5z"/></svg>"#;

const ICON_CLOSE: &str = r#"<svg viewBox="0 0 16 16" fill="currentColor"><path d="M8 8.707l3.646 3.647.708-.707L8.707 8l3.647-3.646-.707-.708L8 7.293 4.354 3.646l-.707.708L7.293 8l-3.646 3.646.707.708L8 8.707z"/></svg>"#;
