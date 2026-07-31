use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::context::MullionContext;
use crate::tree::{PaneData, PaneDirection, PaneId, PaneLayout, PaneRotation, SplitDirection};

/// Host hook used by focus-relative split commands.
///
/// Mullion cannot safely invent application pane ids or pane data. The hook is
/// given the focused pane, requested split axis, and current pane data; it may
/// return the id/data for the new pane or `None` to refuse the split.
pub type PaneSplitFactory<D> =
    Arc<dyn Fn(&PaneId, SplitDirection, &D) -> Option<(PaneId, D)> + Send + Sync>;

/// A focus-relative operation supported by Mullion's command layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaneCommand {
    Focus(PaneDirection),
    FocusNext,
    FocusPrevious,
    FocusFirst,
    FocusLast,
    /// Focus by zero-based traversal index.
    FocusIndex(usize),
    Split(SplitDirection),
    Close,
    Move(PaneDirection),
    Swap(PaneDirection),
    SwapNext,
    SwapPrevious,
    Resize(PaneDirection),
    SetParentSplitDirection(SplitDirection),
    ToggleParentSplitDirection,
    Balance,
    Rotate(PaneRotation),
    ApplyLayout(PaneLayout),
    ToggleZoom,
}

impl PaneCommand {
    /// Static command catalog used by keymaps and command-palette adapters.
    ///
    /// `FocusIndex` is omitted because applications generate those entries from
    /// their live pane list.
    pub fn catalog() -> Vec<Self> {
        use PaneCommand::*;
        use PaneDirection::*;
        vec![
            Focus(Left),
            Focus(Right),
            Focus(Up),
            Focus(Down),
            FocusNext,
            FocusPrevious,
            FocusFirst,
            FocusLast,
            Split(SplitDirection::Horizontal),
            Split(SplitDirection::Vertical),
            Close,
            Move(Left),
            Move(Right),
            Move(Up),
            Move(Down),
            Swap(Left),
            Swap(Right),
            Swap(Up),
            Swap(Down),
            SwapNext,
            SwapPrevious,
            Resize(Left),
            Resize(Right),
            Resize(Up),
            Resize(Down),
            SetParentSplitDirection(SplitDirection::Horizontal),
            SetParentSplitDirection(SplitDirection::Vertical),
            ToggleParentSplitDirection,
            Balance,
            Rotate(PaneRotation::Forward),
            Rotate(PaneRotation::Backward),
            ApplyLayout(PaneLayout::EvenHorizontal),
            ApplyLayout(PaneLayout::EvenVertical),
            ApplyLayout(PaneLayout::MainHorizontal),
            ApplyLayout(PaneLayout::MainVertical),
            ApplyLayout(PaneLayout::Tiled),
            ToggleZoom,
        ]
    }

    /// Stable id suitable for command registries.
    pub fn id(self) -> String {
        use PaneCommand::*;
        match self {
            Focus(direction) => format!("mullion.focus.{}", direction.slug()),
            FocusNext => "mullion.focus.next".into(),
            FocusPrevious => "mullion.focus.previous".into(),
            FocusFirst => "mullion.focus.first".into(),
            FocusLast => "mullion.focus.last".into(),
            FocusIndex(index) => format!("mullion.focus.index.{index}"),
            Split(direction) => format!("mullion.split.{}", direction.slug()),
            Close => "mullion.close".into(),
            Move(direction) => format!("mullion.move.{}", direction.slug()),
            Swap(direction) => format!("mullion.swap.{}", direction.slug()),
            SwapNext => "mullion.swap.next".into(),
            SwapPrevious => "mullion.swap.previous".into(),
            Resize(direction) => format!("mullion.resize.{}", direction.slug()),
            SetParentSplitDirection(direction) => {
                format!("mullion.parent-split.{}", direction.slug())
            }
            ToggleParentSplitDirection => "mullion.parent-split.toggle".into(),
            Balance => "mullion.layout.balance".into(),
            Rotate(PaneRotation::Forward) => "mullion.rotate.forward".into(),
            Rotate(PaneRotation::Backward) => "mullion.rotate.backward".into(),
            ApplyLayout(layout) => format!("mullion.layout.{}", layout.slug()),
            ToggleZoom => "mullion.zoom.toggle".into(),
        }
    }

    /// Human-readable command name.
    pub fn name(self) -> String {
        use PaneCommand::*;
        match self {
            Focus(direction) => format!("Focus Pane {}", direction.label()),
            FocusNext => "Focus Next Pane".into(),
            FocusPrevious => "Focus Previous Pane".into(),
            FocusFirst => "Focus First Pane".into(),
            FocusLast => "Focus Last Pane".into(),
            FocusIndex(index) => format!("Focus Pane {}", index + 1),
            Split(SplitDirection::Horizontal) => "Split Pane Left/Right".into(),
            Split(SplitDirection::Vertical) => "Split Pane Top/Bottom".into(),
            Close => "Close Focused Pane".into(),
            Move(direction) => format!("Move Pane {}", direction.label()),
            Swap(direction) => format!("Swap with Pane {}", direction.label()),
            SwapNext => "Swap with Next Pane".into(),
            SwapPrevious => "Swap with Previous Pane".into(),
            Resize(direction) => format!("Grow Pane {}", direction.label()),
            SetParentSplitDirection(SplitDirection::Horizontal) => {
                "Set Parent Split Left/Right".into()
            }
            SetParentSplitDirection(SplitDirection::Vertical) => {
                "Set Parent Split Top/Bottom".into()
            }
            ToggleParentSplitDirection => "Toggle Parent Split Direction".into(),
            Balance => "Balance Pane Splits".into(),
            Rotate(PaneRotation::Forward) => "Rotate Panes Forward".into(),
            Rotate(PaneRotation::Backward) => "Rotate Panes Backward".into(),
            ApplyLayout(layout) => format!("Apply {} Layout", layout.label()),
            ToggleZoom => "Toggle Focused Pane Zoom".into(),
        }
    }

    /// Command group used by palette integrations.
    pub fn group(self) -> &'static str {
        use PaneCommand::*;
        match self {
            Focus(..) | FocusNext | FocusPrevious | FocusFirst | FocusLast | FocusIndex(..) => {
                "Mullion · Focus"
            }
            Split(..) | Close => "Mullion · Pane",
            Move(..) | Swap(..) | SwapNext | SwapPrevious | Rotate(..) => "Mullion · Arrange",
            Resize(..) => "Mullion · Resize",
            SetParentSplitDirection(..)
            | ToggleParentSplitDirection
            | Balance
            | ApplyLayout(..) => "Mullion · Layout",
            ToggleZoom => "Mullion · View",
        }
    }

    /// Concise behavior description.
    pub fn description(self) -> &'static str {
        use PaneCommand::*;
        match self {
            Focus(..) => "Focus the nearest pane in this direction",
            FocusNext | FocusPrevious => "Cycle focus through the pane layout",
            FocusFirst | FocusLast | FocusIndex(..) => "Focus a pane by layout order",
            Split(..) => "Create and focus a pane beside the focused pane",
            Close => "Close the focused pane and focus an adjacent pane",
            Move(..) => "Move the focused pane beside its directional neighbor",
            Swap(..) | SwapNext | SwapPrevious => {
                "Exchange panes without changing the split topology"
            }
            Resize(..) => "Grow the focused pane toward its nearest boundary",
            SetParentSplitDirection(..) => "Set the focused pane's parent split axis",
            ToggleParentSplitDirection => "Flip the focused pane's parent split axis",
            Balance => "Reset every split ratio to an equal half",
            Rotate(..) => "Rotate panes through the existing layout slots",
            ApplyLayout(..) => "Rebuild the split topology using a standard layout",
            ToggleZoom => "Temporarily fill Mullion with the focused pane",
        }
    }
}

trait DirectionMetadata {
    fn slug(self) -> &'static str;
    fn label(self) -> &'static str;
}

impl DirectionMetadata for PaneDirection {
    fn slug(self) -> &'static str {
        match self {
            PaneDirection::Left => "left",
            PaneDirection::Right => "right",
            PaneDirection::Up => "up",
            PaneDirection::Down => "down",
        }
    }

    fn label(self) -> &'static str {
        match self {
            PaneDirection::Left => "Left",
            PaneDirection::Right => "Right",
            PaneDirection::Up => "Up",
            PaneDirection::Down => "Down",
        }
    }
}

impl DirectionMetadata for SplitDirection {
    fn slug(self) -> &'static str {
        match self {
            SplitDirection::Horizontal => "horizontal",
            SplitDirection::Vertical => "vertical",
        }
    }

    fn label(self) -> &'static str {
        match self {
            SplitDirection::Horizontal => "Horizontal",
            SplitDirection::Vertical => "Vertical",
        }
    }
}

impl PaneLayout {
    fn slug(self) -> &'static str {
        match self {
            PaneLayout::EvenHorizontal => "even-horizontal",
            PaneLayout::EvenVertical => "even-vertical",
            PaneLayout::MainHorizontal => "main-horizontal",
            PaneLayout::MainVertical => "main-vertical",
            PaneLayout::Tiled => "tiled",
        }
    }

    fn label(self) -> &'static str {
        match self {
            PaneLayout::EvenHorizontal => "Even Horizontal",
            PaneLayout::EvenVertical => "Even Vertical",
            PaneLayout::MainHorizontal => "Main Horizontal",
            PaneLayout::MainVertical => "Main Vertical",
            PaneLayout::Tiled => "Tiled",
        }
    }
}

/// Why a pane command could not be applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneCommandError {
    NoFocusedPane,
    NoNeighbor,
    SplitUnavailable,
    SplitRefused,
    CannotCloseLastPane,
    InvalidPaneIndex,
    NotApplicable,
}

impl fmt::Display for PaneCommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            PaneCommandError::NoFocusedPane => "the layout has no focused pane",
            PaneCommandError::NoNeighbor => "there is no pane in that direction",
            PaneCommandError::SplitUnavailable => "no split-pane factory is configured",
            PaneCommandError::SplitRefused => "the split-pane factory refused the split",
            PaneCommandError::CannotCloseLastPane => "the last pane cannot be closed",
            PaneCommandError::InvalidPaneIndex => "there is no pane at that index",
            PaneCommandError::NotApplicable => "the command does not apply to this layout",
        };
        f.write_str(message)
    }
}

impl std::error::Error for PaneCommandError {}

pub type PaneCommandResult = Result<(), PaneCommandError>;

/// Executes [`PaneCommand`] values against a [`MullionContext`].
#[derive(Clone)]
pub struct MullionCommands<D: PaneData> {
    context: MullionContext<D>,
    split_factory: Option<PaneSplitFactory<D>>,
    resize_step: f64,
}

impl<D: PaneData + Send + Sync> MullionCommands<D> {
    pub fn new(context: MullionContext<D>) -> Self {
        Self {
            context,
            split_factory: None,
            resize_step: 0.05,
        }
    }

    /// Configure the host hook used by [`PaneCommand::Split`].
    pub fn with_split_factory(mut self, factory: PaneSplitFactory<D>) -> Self {
        self.split_factory = Some(factory);
        self
    }

    /// Convenience builder that wraps a closure as a [`PaneSplitFactory`].
    pub fn with_split_factory_fn(
        mut self,
        factory: impl Fn(&PaneId, SplitDirection, &D) -> Option<(PaneId, D)> + Send + Sync + 'static,
    ) -> Self {
        self.split_factory = Some(Arc::new(factory));
        self
    }

    /// Set the ratio delta used by directional resize commands.
    pub fn with_resize_step(mut self, step: f64) -> Self {
        if step.is_finite() && step > 0.0 {
            self.resize_step = step;
        }
        self
    }

    pub fn context(&self) -> &MullionContext<D> {
        &self.context
    }

    /// Whether focus-relative split commands have a host factory to call.
    pub fn can_split(&self) -> bool {
        self.split_factory.is_some()
    }

    pub fn execute(&self, command: PaneCommand) -> PaneCommandResult {
        use PaneCommand::*;
        match command {
            Focus(direction) => self
                .context
                .focus_neighbor(direction)
                .then_some(())
                .ok_or(PaneCommandError::NoNeighbor),
            FocusNext => self.cycle_focus(1),
            FocusPrevious => self.cycle_focus(-1),
            FocusFirst => self.focus_index(0),
            FocusLast => {
                let count = self.context.pane_ids().len();
                count
                    .checked_sub(1)
                    .ok_or(PaneCommandError::NoFocusedPane)
                    .and_then(|index| self.focus_index(index))
            }
            FocusIndex(index) => self.focus_index(index),
            Split(direction) => self.split(direction),
            Close => self.close(),
            Move(direction) => self.move_pane(direction),
            Swap(direction) => self.swap_direction(direction),
            SwapNext => self.swap_cycle(1),
            SwapPrevious => self.swap_cycle(-1),
            Resize(direction) => {
                let focused = self.focused()?;
                self.context
                    .resize_pane_toward(&focused, direction, self.resize_step)
                    .then_some(())
                    .ok_or(PaneCommandError::NotApplicable)
            }
            SetParentSplitDirection(direction) => {
                let focused = self.focused()?;
                self.context
                    .try_change_split_direction(&focused, direction)
                    .then_some(())
                    .ok_or(PaneCommandError::NotApplicable)
            }
            ToggleParentSplitDirection => {
                let focused = self.focused()?;
                self.context
                    .toggle_parent_split_direction(&focused)
                    .then_some(())
                    .ok_or(PaneCommandError::NotApplicable)
            }
            Balance => self
                .context
                .balance_splits()
                .then_some(())
                .ok_or(PaneCommandError::NotApplicable),
            Rotate(rotation) => self
                .context
                .rotate_panes(rotation)
                .then_some(())
                .ok_or(PaneCommandError::NotApplicable),
            ApplyLayout(layout) => self
                .context
                .apply_layout(layout)
                .then_some(())
                .ok_or(PaneCommandError::NotApplicable),
            ToggleZoom => self
                .context
                .toggle_zoom()
                .then_some(())
                .ok_or(PaneCommandError::NoFocusedPane),
        }
    }

    fn focused(&self) -> Result<PaneId, PaneCommandError> {
        self.context
            .focused_pane_id()
            .ok_or(PaneCommandError::NoFocusedPane)
    }

    fn cycle_focus(&self, offset: isize) -> PaneCommandResult {
        self.context
            .cycle_focus(offset)
            .then_some(())
            .ok_or(PaneCommandError::NoFocusedPane)
    }

    fn focus_index(&self, index: usize) -> PaneCommandResult {
        self.context
            .focus_pane_at(index)
            .then_some(())
            .ok_or(PaneCommandError::InvalidPaneIndex)
    }

    fn split(&self, direction: SplitDirection) -> PaneCommandResult {
        let focused = self.focused()?;
        let data = self
            .context
            .pane_data(&focused)
            .ok_or(PaneCommandError::NoFocusedPane)?;
        let factory = self
            .split_factory
            .as_ref()
            .ok_or(PaneCommandError::SplitUnavailable)?;
        let (new_id, new_data) =
            factory(&focused, direction, &data).ok_or(PaneCommandError::SplitRefused)?;
        self.context
            .try_split_pane(&focused, direction, new_id, new_data)
            .then_some(())
            .ok_or(PaneCommandError::SplitRefused)
    }

    fn close(&self) -> PaneCommandResult {
        if self.context.pane_ids().len() <= 1 {
            return Err(PaneCommandError::CannotCloseLastPane);
        }
        let focused = self.focused()?;
        self.context
            .close_pane(&focused)
            .map(|_| ())
            .ok_or(PaneCommandError::NotApplicable)
    }

    fn move_pane(&self, direction: PaneDirection) -> PaneCommandResult {
        let focused = self.focused()?;
        let neighbor = self
            .context
            .pane_neighbor(&focused, direction)
            .ok_or(PaneCommandError::NoNeighbor)?;
        self.context
            .try_move_pane(&focused, &neighbor, direction.drop_edge())
            .then(|| {
                self.context.focus_pane(&focused);
            })
            .ok_or(PaneCommandError::NotApplicable)
    }

    fn swap_direction(&self, direction: PaneDirection) -> PaneCommandResult {
        let focused = self.focused()?;
        let neighbor = self
            .context
            .pane_neighbor(&focused, direction)
            .ok_or(PaneCommandError::NoNeighbor)?;
        self.context
            .swap_panes(&focused, &neighbor)
            .then_some(())
            .ok_or(PaneCommandError::NotApplicable)
    }

    fn swap_cycle(&self, offset: isize) -> PaneCommandResult {
        let focused = self.focused()?;
        let ids = self.context.pane_ids();
        if ids.len() < 2 {
            return Err(PaneCommandError::NoNeighbor);
        }
        let index = ids
            .iter()
            .position(|id| id == &focused)
            .ok_or(PaneCommandError::NoFocusedPane)?;
        let neighbor = (index as isize + offset).rem_euclid(ids.len() as isize) as usize;
        self.context
            .swap_panes(&focused, &ids[neighbor])
            .then_some(())
            .ok_or(PaneCommandError::NotApplicable)
    }
}

#[cfg(test)]
mod tests {
    use leptos::prelude::{GetUntracked, Owner};
    use serde::{Deserialize, Serialize};

    use super::*;
    use crate::components::activity_bar::{ActivityBarBehavior, ActivityBarStyle};
    use crate::components::drop_overlay::DropOverlayStyle;
    use crate::components::mullion_root::MullionStyle;
    use crate::components::pane_header::HeaderStyle;
    use crate::components::pane_view::PaneStyle;
    use crate::components::split_handle::SplitHandleStyle;
    use crate::theme::MullionTheme;
    use crate::tree::PaneNode;

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Data(u32);

    fn context(tree: PaneNode<Data>) -> MullionContext<Data> {
        MullionContext::new(
            tree,
            Vec::new(),
            Vec::new(),
            |_| {},
            MullionTheme::default(),
            MullionStyle::default(),
            ActivityBarStyle::default(),
            SplitHandleStyle::default(),
            PaneStyle::default(),
            DropOverlayStyle::default(),
            HeaderStyle::default(),
            ActivityBarBehavior::default(),
            None,
            None,
            None,
            true,
            None,
            None,
            None,
            None,
            None,
        )
    }

    fn two_panes() -> PaneNode<Data> {
        PaneNode::Split {
            direction: SplitDirection::Horizontal,
            ratio: 0.5,
            first: Box::new(PaneNode::leaf(PaneId::new("left"), Data(1))),
            second: Box::new(PaneNode::leaf(PaneId::new("right"), Data(2))),
        }
    }

    #[test]
    fn commands_route_through_durable_focus() {
        let owner = Owner::new();
        owner.set();
        let context = context(two_panes());
        let commands = MullionCommands::new(context.clone());

        assert_eq!(context.focused_pane_id(), Some(PaneId::new("left")));
        commands
            .execute(PaneCommand::Focus(PaneDirection::Right))
            .unwrap();
        assert_eq!(context.focused_pane_id(), Some(PaneId::new("right")));
        commands.execute(PaneCommand::ToggleZoom).unwrap();
        assert_eq!(
            context.zoomed_pane.get_untracked(),
            Some(PaneId::new("right"))
        );
        commands.execute(PaneCommand::FocusPrevious).unwrap();
        assert_eq!(context.focused_pane_id(), Some(PaneId::new("left")));
        assert_eq!(
            context.zoomed_pane.get_untracked(),
            Some(PaneId::new("left"))
        );
    }

    #[test]
    fn split_requires_and_uses_the_host_factory() {
        let owner = Owner::new();
        owner.set();
        let context = context(PaneNode::leaf(PaneId::new("one"), Data(7)));
        let unavailable = MullionCommands::new(context.clone());
        assert_eq!(
            unavailable.execute(PaneCommand::Split(SplitDirection::Vertical)),
            Err(PaneCommandError::SplitUnavailable)
        );

        let commands = unavailable.with_split_factory_fn(|target, direction, data| {
            assert_eq!(target, &PaneId::new("one"));
            assert_eq!(direction, SplitDirection::Vertical);
            Some((PaneId::new("two"), Data(data.0 + 1)))
        });
        commands
            .execute(PaneCommand::Split(SplitDirection::Vertical))
            .unwrap();
        assert_eq!(
            context.pane_ids(),
            vec![PaneId::new("one"), PaneId::new("two")]
        );
        assert_eq!(context.pane_data(&PaneId::new("two")), Some(Data(8)));
        assert_eq!(context.focused_pane_id(), Some(PaneId::new("two")));
    }

    #[test]
    fn closing_focuses_the_remaining_neighbor() {
        let owner = Owner::new();
        owner.set();
        let context = context(two_panes());
        let commands = MullionCommands::new(context.clone());
        commands.execute(PaneCommand::Close).unwrap();
        assert_eq!(context.pane_ids(), vec![PaneId::new("right")]);
        assert_eq!(context.focused_pane_id(), Some(PaneId::new("right")));
        assert_eq!(
            commands.execute(PaneCommand::Close),
            Err(PaneCommandError::CannotCloseLastPane)
        );
    }
}
