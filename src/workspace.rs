use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::tree::{PaneData, PaneNode};

/// A unique identifier for a workspace.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkspaceId(pub String);

/// A named pane layout that can be switched to.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct Workspace<D: PaneData> {
    pub id: WorkspaceId,
    pub name: String,
    pub tree: PaneNode<D>,
}

/// Manages multiple workspaces with reactive switching.
///
/// The consumer creates this, provides it via context or passes it to
/// `<WorkspaceSwitcher>`. Switching workspaces updates the `MullionContext`
/// tree via `set_tree`.
#[derive(Clone)]
pub struct WorkspaceManager<D: PaneData + Send + Sync> {
    workspaces: RwSignal<Vec<Workspace<D>>>,
    active: RwSignal<WorkspaceId>,
}

impl<D: PaneData + Send + Sync> WorkspaceManager<D> {
    pub fn new(workspaces: Vec<Workspace<D>>, active: WorkspaceId) -> Self {
        WorkspaceManager {
            workspaces: RwSignal::new(workspaces),
            active: RwSignal::new(active),
        }
    }

    /// Get the currently active workspace id.
    pub fn active_id(&self) -> WorkspaceId {
        self.active.get()
    }

    /// Get the active workspace id as a signal.
    pub fn active_signal(&self) -> RwSignal<WorkspaceId> {
        self.active
    }

    /// List all workspaces.
    pub fn list(&self) -> Vec<Workspace<D>> {
        self.workspaces.get()
    }

    /// Get the workspaces signal for reactive rendering.
    pub fn workspaces_signal(&self) -> RwSignal<Vec<Workspace<D>>> {
        self.workspaces
    }

    /// Switch to a workspace by id. Returns the workspace's tree if found.
    pub fn switch_to(&self, id: &WorkspaceId) -> Option<PaneNode<D>> {
        let tree = self.workspaces.with(|ws| {
            ws.iter().find(|w| w.id == *id).map(|w| w.tree.clone())
        });
        if tree.is_some() {
            self.active.set(id.clone());
        }
        tree
    }

    /// Add a new workspace.
    pub fn add(&self, workspace: Workspace<D>) {
        self.workspaces.update(|ws| ws.push(workspace));
    }

    /// Remove a workspace by id. Cannot remove the active workspace.
    pub fn remove(&self, id: &WorkspaceId) -> bool {
        if self.active.get_untracked() == *id {
            return false;
        }
        let mut removed = false;
        self.workspaces.update(|ws| {
            if let Some(pos) = ws.iter().position(|w| w.id == *id) {
                ws.remove(pos);
                removed = true;
            }
        });
        removed
    }

    /// Update a workspace's tree (e.g., save the current layout back).
    pub fn update_tree(&self, id: &WorkspaceId, tree: PaneNode<D>) {
        self.workspaces.update(|ws| {
            if let Some(w) = ws.iter_mut().find(|w| w.id == *id) {
                w.tree = tree;
            }
        });
    }

    /// Rename a workspace.
    pub fn rename(&self, id: &WorkspaceId, new_name: String) {
        self.workspaces.update(|ws| {
            if let Some(w) = ws.iter_mut().find(|w| w.id == *id) {
                w.name = new_name;
            }
        });
    }
}
