use okshell_services::hyprland_service;
use std::sync::Arc;
use tracing::error;
use wayle_hyprland::{Workspace, WorkspaceInfo};

pub fn get_active_workspaces() -> Vec<WorkspaceInfo> {
    let hyprland = hyprland_service();
    let mut active_workspaces: Vec<WorkspaceInfo> = Vec::new();
    for monitor in hyprland.monitors.get() {
        active_workspaces.push(monitor.active_workspace.get());
    }
    active_workspaces
}

pub fn is_an_active_workspace(workspace: &Arc<Workspace>) -> bool {
    get_active_workspaces()
        .iter()
        .find(|p| p.id == workspace.id.get())
        .is_some()
}

pub fn go_up_workspace() {
    let hyprland = hyprland_service();
    tokio::spawn(async move {
        if let Err(e) = hyprland.dispatch("workspace r-1").await {
            error!(error = %e, "Failed to switch workspace");
        }
    });
}

pub fn go_down_workspace() {
    let hyprland = hyprland_service();
    tokio::spawn(async move {
        if let Err(e) = hyprland.dispatch("workspace r+1").await {
            error!(error = %e, "Failed to switch workspace");
        }
    });
}
