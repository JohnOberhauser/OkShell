use okshell_common::watch_cancellable;
use okshell_services::hyprland_service;
use relm4::{Component, ComponentSender};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
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

pub fn get_active_workspace_for_connector(connector: &String) -> Option<Arc<Workspace>> {
    let hyprland = hyprland_service();
    let active_workspaces = get_active_workspaces();
    let workspaces = hyprland.workspaces.get();

    let workspace = workspaces
        .iter()
        .filter(|w| w.monitor.get() == *connector)
        .find(|w| active_workspaces.iter().any(|aw| aw.id == w.id.get()));

    return workspace.cloned();
}

pub fn go_up_workspace() {
    let hyprland = hyprland_service();
    tokio::spawn(async move {
        if let Err(e) = hyprland
            .dispatch("hl.dsp.focus({ workspace = \"r-1\" })")
            .await
        {
            error!(error = %e, "Failed to switch workspace");
        }
    });
}

pub fn go_down_workspace() {
    let hyprland = hyprland_service();
    tokio::spawn(async move {
        if let Err(e) = hyprland
            .dispatch("hl.dsp.focus({ workspace = \"r+1\" })")
            .await
        {
            error!(error = %e, "Failed to switch workspace");
        }
    });
}

pub fn spawn_workspace_layout_watcher<C>(
    workspace: &Arc<Workspace>,
    cancellation_token: CancellationToken,
    sender: &ComponentSender<C>,
    map_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let layout = workspace.tiled_layout.clone();
    watch_cancellable!(sender, cancellation_token, [layout.watch()], |out| {
        let _ = out.send(map_state());
    });
}
