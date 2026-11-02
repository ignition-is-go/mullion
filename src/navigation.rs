//! Deliberate view navigation, separate from layout mutations and server updates.
//!
//! Browser entries project activity selections only. Replay compares adjacent
//! entries and patches changed addresses when the live value still matches the
//! departure entry. This preserves both current layout and later programmatic
//! activity writes. A destination alone cannot undo navigation in another pane.

use std::sync::Arc;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    ActivityId, MullionContext, PaneData, PaneId, PaneNode, WorkspaceId, WorkspaceManager,
};

mod browser;
pub use browser::PaneBrowserHistory;

/// Outcome of an explicit navigation request. Rejected requests never mutate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavigationResult {
    Changed,
    Unchanged,
    MissingWorkspace,
    MissingPane,
    UnavailableActivity,
}

/// Failures and conflicts reported by the optional browser bridge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavigationDiagnostic {
    Browser(String),
    ReplayRejected(NavigationResult),
    Conflict {
        workspace: Option<WorkspaceId>,
        pane: PaneId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Selection {
    workspace: Option<WorkspaceId>,
    pane: PaneId,
    activity: Option<ActivityId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Projection {
    active: Option<WorkspaceId>,
    target: Option<PaneId>,
    selections: Vec<Selection>,
}

#[cfg(any(test, target_arch = "wasm32"))]
impl Projection {
    fn selection(&self, workspace: &Option<WorkspaceId>, pane: &PaneId) -> Option<&Selection> {
        self.selections
            .iter()
            .find(|s| s.workspace == *workspace && s.pane == *pane)
    }
}

#[cfg(any(test, target_arch = "wasm32"))]
#[derive(Clone, Debug)]
struct Change {
    before: Selection,
    after: Option<ActivityId>,
}

#[cfg(any(test, target_arch = "wasm32"))]
fn differences(before: &Projection, after: &Projection) -> Vec<Change> {
    after
        .selections
        .iter()
        .filter_map(|next| {
            let previous = before.selection(&next.workspace, &next.pane)?;
            (previous.activity != next.activity).then(|| Change {
                before: previous.clone(),
                after: next.activity.clone(),
            })
        })
        .collect()
}

// Refresh the departure value only for addresses this navigation changes.
// Untouched live selections must never become historical navigation changes.
#[cfg(any(test, target_arch = "wasm32"))]
fn project_navigation(
    mut baseline: Projection,
    before: &Projection,
    after: &Projection,
) -> (Projection, Projection) {
    let changes = differences(before, after);
    for change in &changes {
        if let Some(selection) = baseline
            .selections
            .iter_mut()
            .find(|s| s.workspace == change.before.workspace && s.pane == change.before.pane)
        {
            selection.activity = change.before.activity.clone();
        } else {
            baseline.selections.push(change.before.clone());
        }
    }
    let mut next = baseline.clone();
    for change in changes {
        if let Some(selection) = next
            .selections
            .iter_mut()
            .find(|s| s.workspace == change.before.workspace && s.pane == change.before.pane)
        {
            selection.activity = change.after;
        }
    }
    next.active = after.active.clone();
    next.target = after.target.clone();
    (baseline, next)
}

#[cfg(any(test, target_arch = "wasm32"))]
fn apply_changes<D: PaneData>(
    tree: &mut PaneNode<D>,
    workspace: &Option<WorkspaceId>,
    changes: &[Change],
) -> (bool, Vec<PaneId>) {
    let mut changed = false;
    let mut conflicts = Vec::new();
    for change in changes.iter().filter(|c| c.before.workspace == *workspace) {
        if let Some(PaneNode::Leaf {
            active_activity, ..
        }) = tree.find_mut(&change.before.pane)
        {
            if *active_activity == change.before.activity {
                *active_activity = change.after.clone();
                changed = true;
            } else {
                conflicts.push(change.before.pane.clone());
            }
        }
    }
    (changed, conflicts)
}

type Observer = Arc<dyn Fn(Projection, Projection) + Send + Sync>;

/// Coordinates pane navigation and optional workspace switching.
///
/// Provide a clone through Leptos context to enable `PaneLink`, the activity
/// rail, and `WorkspaceSwitcher`. Creating this handle never owns browser history.
#[derive(Clone)]
pub struct PaneNavigation<D: PaneData + Send + Sync> {
    ctx: MullionContext<D>,
    manager: Option<WorkspaceManager<D>>,
    observer: StoredValue<Option<Observer>>,
}

impl<D: PaneData + Send + Sync> PaneNavigation<D> {
    pub fn single(ctx: MullionContext<D>) -> Self {
        Self {
            ctx,
            manager: None,
            observer: StoredValue::new(None),
        }
    }

    pub fn with_workspaces(ctx: MullionContext<D>, manager: WorkspaceManager<D>) -> Self {
        Self {
            ctx,
            manager: Some(manager),
            observer: StoredValue::new(None),
        }
    }

    fn active(&self) -> Option<WorkspaceId> {
        self.manager
            .as_ref()
            .map(|m| m.active_signal().get_untracked())
    }

    fn projection(&self, target: Option<PaneId>) -> Projection {
        let active = self.active();
        let mut selections = Vec::new();
        let mut collect = |workspace: Option<WorkspaceId>, tree: &PaneNode<D>| {
            for pane in tree.leaf_ids() {
                if let Some(PaneNode::Leaf {
                    active_activity, ..
                }) = tree.find(&pane)
                {
                    selections.push(Selection {
                        workspace: workspace.clone(),
                        pane,
                        activity: active_activity.clone(),
                    });
                }
            }
        };
        if let Some(manager) = &self.manager {
            for workspace in manager.workspaces_signal().get_untracked() {
                if active.as_ref() == Some(&workspace.id) {
                    self.ctx
                        .tree
                        .with_untracked(|tree| collect(Some(workspace.id), tree));
                } else {
                    collect(Some(workspace.id), &workspace.tree);
                }
            }
        } else {
            self.ctx.tree.with_untracked(|tree| collect(None, tree));
        }
        Projection {
            active,
            target,
            selections,
        }
    }

    fn validate(
        &self,
        tree: &PaneNode<D>,
        pane: &PaneId,
        activity: &Option<ActivityId>,
    ) -> NavigationResult {
        let Some(PaneNode::Leaf {
            data,
            active_activity,
            ..
        }) = tree.find(pane)
        else {
            return NavigationResult::MissingPane;
        };
        if let Some(activity) = activity {
            if !self
                .ctx
                .activities_for_pane(data)
                .iter()
                .any(|a| a.def.id == *activity)
            {
                return NavigationResult::UnavailableActivity;
            }
        }
        if active_activity == activity {
            NavigationResult::Unchanged
        } else {
            NavigationResult::Changed
        }
    }

    fn notify(&self, before: Projection, target: Option<PaneId>) {
        let observer = self.observer.get_value();
        if let Some(observer) = observer {
            observer(before, self.projection(target));
        }
    }

    /// Select an activity in this exact pane of the currently active workspace.
    pub fn navigate(&self, pane: &PaneId, activity: Option<ActivityId>) -> NavigationResult {
        let result = self
            .ctx
            .tree
            .with_untracked(|tree| self.validate(tree, pane, &activity));
        if result != NavigationResult::Changed {
            return result;
        }
        let before = self.projection(Some(pane.clone()));
        self.ctx.set_active_activity(pane, activity);
        self.notify(before, Some(pane.clone()));
        NavigationResult::Changed
    }

    /// Save the departing workspace and select the current layout of another.
    pub fn switch_workspace(&self, id: &WorkspaceId) -> NavigationResult {
        let Some(manager) = &self.manager else {
            return NavigationResult::MissingWorkspace;
        };
        if self.active().as_ref() == Some(id) {
            return NavigationResult::Unchanged;
        }
        let Some(workspace) = manager
            .workspaces_signal()
            .get_untracked()
            .into_iter()
            .find(|w| w.id == *id)
        else {
            return NavigationResult::MissingWorkspace;
        };
        let before = self.projection(None);
        self.save_current();
        batch(|| {
            manager.switch_to(id);
            self.ctx.set_tree(workspace.tree);
        });
        self.ctx.emit_tree_changed();
        self.notify(before, None);
        NavigationResult::Changed
    }

    fn save_current(&self) {
        if let Some(manager) = &self.manager {
            manager.update_tree(
                &manager.active_signal().get_untracked(),
                self.ctx.tree.get_untracked(),
            );
        }
    }

    #[cfg(any(test, target_arch = "wasm32"))]
    fn restore(
        &self,
        before: &Projection,
        after: &Projection,
    ) -> Result<Vec<NavigationDiagnostic>, NavigationResult> {
        if self.manager.is_some() != after.active.is_some() {
            return Err(NavigationResult::MissingWorkspace);
        }
        let destination = if let Some(manager) = &self.manager {
            manager
                .workspaces_signal()
                .get_untracked()
                .into_iter()
                .find(|w| Some(&w.id) == after.active.as_ref())
                .map(|w| w.tree)
        } else {
            Some(self.ctx.tree.get_untracked())
        };
        let Some(mut destination) = destination else {
            return Err(NavigationResult::MissingWorkspace);
        };
        if after.active == self.active() {
            destination = self.ctx.tree.get_untracked();
        }
        if let Some(pane) = &after.target {
            let Some(selection) = after.selection(&after.active, pane) else {
                return Err(NavigationResult::MissingPane);
            };
            let valid = self.validate(&destination, pane, &selection.activity);
            if !matches!(
                valid,
                NavigationResult::Changed | NavigationResult::Unchanged
            ) {
                return Err(valid);
            }
        }
        let changes = differences(before, after);
        // An activity may have disappeared from registration since this entry.
        // Validate every proposed patch before mutating any workspace.
        for change in &changes {
            let tree = if change.before.workspace == self.active() {
                Some(self.ctx.tree.get_untracked())
            } else {
                self.manager.as_ref().and_then(|m| {
                    m.workspaces_signal()
                        .get_untracked()
                        .into_iter()
                        .find(|w| Some(&w.id) == change.before.workspace.as_ref())
                        .map(|w| w.tree)
                })
            };
            if let Some(tree) = tree {
                if self.validate(&tree, &change.before.pane, &change.after)
                    == NavigationResult::UnavailableActivity
                {
                    return Err(NavigationResult::UnavailableActivity);
                }
            }
        }
        let mut diagnostics = Vec::new();
        self.save_current();
        let switched = self.active() != after.active;
        let mut any_changed = false;
        batch(|| {
            if let Some(manager) = &self.manager {
                let current = self.active();
                for mut workspace in manager.workspaces_signal().get_untracked() {
                    let key = Some(workspace.id.clone());
                    if !changes.iter().any(|c| c.before.workspace == key) {
                        continue;
                    }
                    let (changed, conflicts) = if key == current {
                        let mut result = (false, Vec::new());
                        self.ctx
                            .tree
                            .update(|tree| result = apply_changes(tree, &key, &changes));
                        workspace.tree = self.ctx.tree.get_untracked();
                        result
                    } else {
                        apply_changes(&mut workspace.tree, &key, &changes)
                    };
                    for pane in conflicts {
                        diagnostics.push(NavigationDiagnostic::Conflict {
                            workspace: key.clone(),
                            pane,
                        });
                    }
                    if changed {
                        manager.update_tree(&workspace.id, workspace.tree);
                        any_changed = true;
                    }
                }
                if switched {
                    if let Some(tree) =
                        manager.switch_to(after.active.as_ref().expect("validated workspace"))
                    {
                        self.ctx.set_tree(tree);
                    }
                }
            } else {
                let (changed, conflicts) = apply_changes(&mut destination, &None, &changes);
                for pane in conflicts {
                    diagnostics.push(NavigationDiagnostic::Conflict {
                        workspace: None,
                        pane,
                    });
                }
                if changed {
                    self.ctx.set_tree(destination);
                    any_changed = true;
                }
            }
        });
        if switched || any_changed {
            self.ctx.emit_tree_changed();
        }
        Ok(diagnostics)
    }
}

/// A pane-local view button. It has no URL or new-tab semantics.
#[component]
pub fn PaneLink<D: PaneData + Send + Sync>(
    pane: PaneId,
    activity: ActivityId,
    #[prop(optional, into)] class: String,
    children: Children,
    #[prop(optional)] navigation: Option<PaneNavigation<D>>,
) -> impl IntoView {
    let navigation = navigation.unwrap_or_else(expect_context::<PaneNavigation<D>>);
    view! { <button type="button" class=class on:click=move |_| { navigation.navigate(&pane, Some(activity.clone())); }>{children()}</button> }
}

pub(crate) fn select_activity<D: PaneData + Send + Sync>(
    ctx: &MullionContext<D>,
    navigation: &Option<PaneNavigation<D>>,
    pane: &PaneId,
    activity: ActivityId,
) {
    if let Some(navigation) = navigation {
        navigation.navigate(pane, Some(activity));
    } else {
        ctx.set_active_activity(pane, Some(activity));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ActivityDef, ActivityIcon, ActivityNode, PaneEvent, SplitDirection, Workspace};
    use std::sync::Mutex;

    fn leaf(id: &str, activity: &str) -> PaneNode<String> {
        PaneNode::leaf_with_activity(
            PaneId(id.into()),
            ActivityId::new(activity),
            format!("data-{id}"),
        )
    }

    fn pair() -> PaneNode<String> {
        PaneNode::Split {
            direction: SplitDirection::Horizontal,
            ratio: 0.73,
            first: Box::new(leaf("a", "main")),
            second: Box::new(leaf("b", "main")),
        }
    }

    fn context(
        tree: PaneNode<String>,
        on_event: impl Fn(PaneEvent<String>) + Send + Sync + 'static,
    ) -> MullionContext<String> {
        let items = ["main", "fleet", "logs", "programmatic"]
            .into_iter()
            .map(|id| {
                ActivityNode::activity(ActivityDef::new(
                    ActivityId::new(id),
                    id,
                    ActivityIcon::Class(String::new()),
                    |_| true,
                    |_, _| ().into_any(),
                ))
            })
            .collect();
        MullionContext::new(
            tree,
            items,
            vec![],
            on_event,
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
            Default::default(),
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

    fn activity(tree: &PaneNode<String>, id: &str) -> Option<ActivityId> {
        match tree.find(&PaneId(id.into())).unwrap() {
            PaneNode::Leaf {
                active_activity, ..
            } => active_activity.clone(),
            _ => unreachable!(),
        }
    }

    fn assert_activity(ctx: &MullionContext<String>, id: &str, expected: &str) {
        assert_eq!(
            activity(&ctx.tree.get_untracked(), id),
            Some(ActivityId::new(expected))
        );
    }

    fn entries(nav: &PaneNavigation<String>) -> (Projection, Projection, Projection) {
        let initial = nav.projection(None);
        let before = nav.projection(None);
        nav.navigate(&PaneId("a".into()), Some(ActivityId::new("fleet")));
        let (_, first) = project_navigation(
            initial.clone(),
            &before,
            &nav.projection(Some(PaneId("a".into()))),
        );
        let before = nav.projection(None);
        nav.navigate(&PaneId("b".into()), Some(ActivityId::new("logs")));
        let (_, second) = project_navigation(
            first.clone(),
            &before,
            &nav.projection(Some(PaneId("b".into()))),
        );
        (initial, first, second)
    }

    #[test]
    fn multipane_back_forward_and_multi_entry_jump() {
        Owner::new().with(|| {
            let ctx = context(pair(), |_| {});
            let nav = PaneNavigation::single(ctx.clone());
            let (initial, first, second) = entries(&nav);
            assert!(nav.restore(&second, &first).unwrap().is_empty());
            assert_activity(&ctx, "a", "fleet");
            assert_activity(&ctx, "b", "main");
            nav.restore(&first, &initial).unwrap();
            assert_activity(&ctx, "a", "main");
            assert_activity(&ctx, "b", "main");
            nav.restore(&initial, &second).unwrap();
            assert_activity(&ctx, "a", "fleet");
            assert_activity(&ctx, "b", "logs");
            nav.restore(&second, &initial).unwrap();
            assert_activity(&ctx, "a", "main");
            assert_activity(&ctx, "b", "main");
        });
    }

    #[test]
    fn programmatic_edit_before_later_link_is_not_part_of_history() {
        Owner::new().with(|| {
            let ctx = context(pair(), |_| {});
            let nav = PaneNavigation::single(ctx.clone());
            let initial = nav.projection(None);
            ctx.set_active_activity(&PaneId("b".into()), Some(ActivityId::new("programmatic")));
            let before = nav.projection(None);
            nav.navigate(&PaneId("a".into()), Some(ActivityId::new("fleet")));
            let (_, next) = project_navigation(
                initial.clone(),
                &before,
                &nav.projection(Some(PaneId("a".into()))),
            );
            assert_eq!(differences(&initial, &next).len(), 1);
            nav.restore(&next, &initial).unwrap();
            assert_activity(&ctx, "a", "main");
            assert_activity(&ctx, "b", "programmatic");
        });
    }

    #[test]
    fn a_link_returns_to_the_programmatic_view_it_departed() {
        Owner::new().with(|| {
            let ctx = context(pair(), |_| {});
            let nav = PaneNavigation::single(ctx.clone());
            let initial = nav.projection(None);
            ctx.set_active_activity(&PaneId("a".into()), Some(ActivityId::new("logs")));
            let before = nav.projection(None);
            nav.navigate(&PaneId("a".into()), Some(ActivityId::new("fleet")));
            let (baseline, next) =
                project_navigation(initial, &before, &nav.projection(Some(PaneId("a".into()))));
            nav.restore(&next, &baseline).unwrap();
            assert_activity(&ctx, "a", "logs");
            nav.restore(&baseline, &next).unwrap();
            assert_activity(&ctx, "a", "fleet");
        });
    }

    #[test]
    fn programmatic_edit_after_latest_entry_survives_back_forward_jumps() {
        Owner::new().with(|| {
            let ctx = context(pair(), |_| {});
            let nav = PaneNavigation::single(ctx.clone());
            let (initial, _, second) = entries(&nav);
            ctx.set_active_activity(&PaneId("b".into()), Some(ActivityId::new("programmatic")));
            let diagnostics = nav.restore(&second, &initial).unwrap();
            assert_eq!(
                diagnostics,
                vec![NavigationDiagnostic::Conflict {
                    workspace: None,
                    pane: PaneId("b".into())
                }]
            );
            assert_activity(&ctx, "a", "main");
            assert_activity(&ctx, "b", "programmatic");
            assert_eq!(nav.restore(&initial, &second).unwrap().len(), 1);
            assert_activity(&ctx, "a", "fleet");
            assert_activity(&ctx, "b", "programmatic");
        });
    }

    #[test]
    fn replay_preserves_current_structure_ratios_data_and_new_panes() {
        Owner::new().with(|| {
            let ctx = context(pair(), |_| {});
            let nav = PaneNavigation::single(ctx.clone());
            let (initial, _, second) = entries(&nav);
            ctx.tree.update(|tree| {
                let PaneNode::Split { ratio, second, .. } = tree else {
                    unreachable!()
                };
                *ratio = 0.19;
                **second = PaneNode::Split {
                    direction: SplitDirection::Vertical,
                    ratio: 0.42,
                    first: Box::new(leaf("b", "logs")),
                    second: Box::new(leaf("new", "main")),
                };
                let Some(PaneNode::Leaf { data, .. }) = tree.find_mut(&PaneId("a".into())) else {
                    unreachable!()
                };
                *data = "latest-server-data".into();
            });
            let expected = ctx.tree.get_untracked();
            nav.restore(&second, &initial).unwrap();
            let mut actual = ctx.tree.get_untracked();
            if let Some(PaneNode::Leaf {
                active_activity, ..
            }) = actual.find_mut(&PaneId("a".into()))
            {
                *active_activity = Some(ActivityId::new("fleet"));
            }
            if let Some(PaneNode::Leaf {
                active_activity, ..
            }) = actual.find_mut(&PaneId("b".into()))
            {
                *active_activity = Some(ActivityId::new("logs"));
            }
            assert_eq!(actual, expected);
        });
    }

    #[test]
    fn noops_and_missing_panes_emit_nothing_and_do_not_notify() {
        Owner::new().with(|| {
            let events = Arc::new(Mutex::new(0));
            let copy = events.clone();
            let ctx = context(pair(), move |_| *copy.lock().unwrap() += 1);
            let nav = PaneNavigation::single(ctx.clone());
            nav.observer
                .set_value(Some(Arc::new(|_, _| panic!("unexpected navigation"))));
            ctx.set_active_activity(&PaneId("a".into()), Some(ActivityId::new("main")));
            ctx.set_active_activity(&PaneId("missing".into()), None);
            assert_eq!(
                nav.navigate(&PaneId("a".into()), Some(ActivityId::new("main"))),
                NavigationResult::Unchanged
            );
            assert_eq!(
                nav.navigate(&PaneId("missing".into()), Some(ActivityId::new("main"))),
                NavigationResult::MissingPane
            );
            assert_eq!(
                nav.navigate(&PaneId("a".into()), Some(ActivityId::new("unknown"))),
                NavigationResult::UnavailableActivity
            );
            assert_eq!(
                nav.switch_workspace(&WorkspaceId("absent".into())),
                NavigationResult::MissingWorkspace
            );
            assert_eq!(*events.lock().unwrap(), 0);
        });
    }

    #[test]
    fn missing_required_target_rejects_atomically_and_deleted_other_panes_stay_deleted() {
        Owner::new().with(|| {
            let ctx = context(pair(), |_| {});
            let nav = PaneNavigation::single(ctx.clone());
            let (initial, first, second) = entries(&nav);
            ctx.set_tree(leaf("b", "logs"));
            let unchanged = ctx.tree.get_untracked();
            assert_eq!(
                nav.restore(&second, &first),
                Err(NavigationResult::MissingPane)
            );
            assert_eq!(ctx.tree.get_untracked(), unchanged);
            nav.restore(&second, &initial).unwrap();
            assert_activity(&ctx, "b", "main");
            assert_eq!(
                ctx.tree.get_untracked().leaf_ids(),
                vec![PaneId("b".into())]
            );
        });
    }

    #[test]
    fn newly_introduced_navigation_address_gets_only_current_entry_baseline() {
        Owner::new().with(|| {
            let ctx = context(leaf("a", "main"), |_| {});
            let nav = PaneNavigation::single(ctx.clone());
            let initial = nav.projection(None);
            ctx.set_tree(pair());
            let before = nav.projection(None);
            nav.navigate(&PaneId("b".into()), Some(ActivityId::new("logs")));
            let (baseline, next) = project_navigation(
                initial.clone(),
                &before,
                &nav.projection(Some(PaneId("b".into()))),
            );
            assert!(initial.selection(&None, &PaneId("b".into())).is_none());
            assert_eq!(
                baseline
                    .selection(&None, &PaneId("b".into()))
                    .unwrap()
                    .activity,
                Some(ActivityId::new("main"))
            );
            nav.restore(&next, &baseline).unwrap();
            assert_activity(&ctx, "b", "main");
        });
    }

    #[test]
    fn workspace_replay_scopes_overlapping_pane_ids_and_persists_after_switch() {
        Owner::new().with(|| {
            let manager = WorkspaceManager::new(
                vec![
                    Workspace {
                        id: WorkspaceId("one".into()),
                        name: "One".into(),
                        tree: pair(),
                    },
                    Workspace {
                        id: WorkspaceId("two".into()),
                        name: "Two".into(),
                        tree: pair(),
                    },
                ],
                WorkspaceId("one".into()),
            );
            let event_manager = manager.clone();
            let seen = Arc::new(Mutex::new(Vec::new()));
            let copy = seen.clone();
            let ctx = context(pair(), move |event| {
                if let PaneEvent::TreeChanged { tree } = event {
                    let id = event_manager.active_signal().get_untracked();
                    copy.lock().unwrap().push(id.clone());
                    event_manager.update_tree(&id, tree);
                }
            });
            let nav = PaneNavigation::with_workspaces(ctx.clone(), manager.clone());
            let baseline = nav.projection(None);
            nav.navigate(&PaneId("a".into()), Some(ActivityId::new("fleet")));
            let first = nav.projection(Some(PaneId("a".into())));
            nav.switch_workspace(&WorkspaceId("two".into()));
            let switched = nav.projection(None);
            nav.navigate(&PaneId("a".into()), Some(ActivityId::new("logs")));
            let latest = nav.projection(Some(PaneId("a".into())));
            seen.lock().unwrap().clear();
            nav.restore(&latest, &first).unwrap();
            assert_eq!(
                manager.active_signal().get_untracked(),
                WorkspaceId("one".into())
            );
            assert_activity(&ctx, "a", "fleet");
            assert_eq!(
                activity(&manager.workspaces_signal().get_untracked()[1].tree, "a"),
                Some(ActivityId::new("main"))
            );
            assert_eq!(*seen.lock().unwrap(), vec![WorkspaceId("one".into())]);
            nav.restore(&first, &baseline).unwrap();
            assert_activity(&ctx, "a", "main");
            nav.restore(&baseline, &switched).unwrap();
            assert_eq!(
                manager.active_signal().get_untracked(),
                WorkspaceId("two".into())
            );
            assert_activity(&ctx, "a", "main");
            let before = ctx.tree.get_untracked();
            let mut missing = switched;
            missing.active = Some(WorkspaceId("gone".into()));
            assert_eq!(
                nav.restore(&baseline, &missing),
                Err(NavigationResult::MissingWorkspace)
            );
            assert_eq!(ctx.tree.get_untracked(), before);
        });
    }

    #[test]
    fn browser_replay_does_not_notify_user_navigation_observer() {
        Owner::new().with(|| {
            let ctx = context(pair(), |_| {});
            let nav = PaneNavigation::single(ctx);
            let (initial, _, second) = entries(&nav);
            nav.observer
                .set_value(Some(Arc::new(|_, _| panic!("replay must not push"))));
            nav.restore(&second, &initial).unwrap();
        });
    }
}
