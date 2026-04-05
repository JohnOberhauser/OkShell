use std::sync::Arc;
use futures::StreamExt;
use relm4::{
    gtk,
    gtk::prelude::*,
    Component,
    ComponentController,
    ComponentParts,
    ComponentSender,
    Controller
};
use relm4::gtk::{Orientation, RevealerTransitionType};
use wayle_hyprland::{HyprlandEvent, MonitorId, Workspace, WorkspaceId};
use okshell_services::hyprland_service;
use crate::bars::bar_widgets::hyprland_workspace::{HyprlandWorkspaceInput, HyprlandWorkspaceModel};
use okshell_common::dynamic_box::dynamic_box::{DynamicBoxFactory, DynamicBoxInit, DynamicBoxInput, DynamicBoxModel};
use okshell_common::dynamic_box::generic_widget_controller::{GenericWidgetController, GenericWidgetControllerExtSafe};
use okshell_common::dynamic_box::simple_widget_controller::SimpleWidgetController;
use okshell_utils::hover_scroll::attach_hover_scroll;
use okshell_utils::hyprland::{get_active_workspaces, go_down_workspace, go_up_workspace};

#[derive(Clone, Debug)]
pub enum WsRow {
    Divider(MonitorId),
    Workspace(Arc<Workspace>),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum WsRowKey {
    Divider(MonitorId),
    Workspace(WorkspaceId),
}

pub(crate) struct HyprlandWorkspacesModel {
    dynamic_box: Controller<DynamicBoxModel<WsRow, WsRowKey>>,
    orientation: Orientation,
}

#[derive(Debug)]
pub(crate) enum HyprlandWorkspacesInput {}

#[derive(Debug)]
pub(crate) enum HyprlandWorkspacesOutput {}

pub(crate) struct HyprlandWorkspacesInit {
    pub(crate) orientation: Orientation,
}

#[derive(Debug)]
pub(crate) enum HyprlandWorkspacesCommandOutput {
    WorkspacesChanged,
    ActiveWorkspaceChanged,
}

#[relm4::component(pub)]
impl Component for HyprlandWorkspacesModel {
    type CommandOutput = HyprlandWorkspacesCommandOutput;
    type Input = HyprlandWorkspacesInput;
    type Output = HyprlandWorkspacesOutput;
    type Init = HyprlandWorkspacesInit;

    view! {
        #[root]
        #[name = "workspace_box"]
        gtk::Box {
            add_css_class: "hyprland-workspaces-bar-widget",
            set_hexpand: model.orientation == Orientation::Vertical,
            set_vexpand: model.orientation == Orientation::Horizontal,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        Self::spawn_main_watcher(&sender);

        let divider_orientation = if params.orientation == Orientation::Horizontal {
            Orientation::Vertical
        } else {
            Orientation::Horizontal
        };

        let factory = DynamicBoxFactory::<WsRow, WsRowKey> {
            id: Box::new(|item| match item {
                WsRow::Divider(monitor_id) => WsRowKey::Divider(monitor_id.clone()),
                WsRow::Workspace(workspace) => WsRowKey::Workspace(workspace.id.get()),
            }),
            create: Box::new(move |item| match item {
                WsRow::Workspace(workspace) => {
                    let controller: Controller<HyprlandWorkspaceModel> =
                        HyprlandWorkspaceModel::builder()
                            .launch(workspace.clone().into())
                            .detach();
                    Box::new(controller) as Box<dyn GenericWidgetController>
                }
                WsRow::Divider(_) => {
                    let sep = gtk::Separator::new(divider_orientation);
                    sep.add_css_class("workspace-divider");
                    Box::new(SimpleWidgetController::new(sep.upcast()))
                }
            }),
            update: None,
        };

        let transition_type = if params.orientation == Orientation::Horizontal {
            RevealerTransitionType::SwingLeft
        } else {
            RevealerTransitionType::SwingUp
        };

        let dynamic: Controller<DynamicBoxModel<WsRow, WsRowKey>> =
            DynamicBoxModel::builder()
                .launch(DynamicBoxInit{
                    factory,
                    orientation: params.orientation,
                    spacing: 0,
                    transition_type,
                    transition_duration_ms: 200,
                    reverse: false,
                    retain_entries: false,
                })
                .detach();

        let model = HyprlandWorkspacesModel {
            dynamic_box: dynamic,
            orientation: params.orientation,
        };

        let widgets = view_output!();

        widgets.workspace_box.append(model.dynamic_box.widget());

        let hyprland = hyprland_service();
        let workspaces = hyprland.workspaces.get();

        let workspaces = Self::workspaces_with_dividers(workspaces);

        model.dynamic_box.sender().send(DynamicBoxInput::SetItems(workspaces)).unwrap();

        let _handles = attach_hover_scroll(&widgets.workspace_box, move |_dx, dy, _hovered, _| {
            if dy < 0.0 {
                go_up_workspace()
            } else if dy > 0.0 {
                go_down_workspace()
            }
        });

        ComponentParts { model, widgets }
    }

    fn update_cmd_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            HyprlandWorkspacesCommandOutput::WorkspacesChanged => {
                let hyprland = hyprland_service();
                let workspaces = hyprland.workspaces.get();

                let workspaces = Self::workspaces_with_dividers(workspaces);

                self.dynamic_box.sender().send(DynamicBoxInput::SetItems(workspaces)).unwrap();
            }
            HyprlandWorkspacesCommandOutput::ActiveWorkspaceChanged => {
                let active_workspaces = get_active_workspaces();

                self.dynamic_box.model().for_each_entry(|_, entry| {
                    if let Some(ctrl) = entry
                        .controller
                        .as_ref()
                        .downcast_ref::<Controller<HyprlandWorkspaceModel>>()
                    {
                        let _ = ctrl
                            .sender()
                            .send(HyprlandWorkspaceInput::ActiveUpdate(active_workspaces.clone()));
                    }
                })
            }
        }
    }
}

impl HyprlandWorkspacesModel {
    fn spawn_main_watcher(
        sender: &ComponentSender<Self>,
    ) {
        sender.command(move |out, shutdown| {
            async move {
                let hyprland = hyprland_service();
                let mut events = hyprland.events();
                let shutdown_fut = shutdown.wait();
                tokio::pin!(shutdown_fut);

                loop {
                    tokio::select! {
                        () = &mut shutdown_fut => return,
                        event = events.next() => {
                            let Some(event) = event else { continue; };
                            match event {
                                HyprlandEvent::WorkspaceV2 { .. } => {
                                    let _ = out.send(HyprlandWorkspacesCommandOutput::ActiveWorkspaceChanged);
                                }
                                HyprlandEvent::CreateWorkspaceV2 { .. }
                                | HyprlandEvent::DestroyWorkspaceV2 { .. }
                                | HyprlandEvent::MoveWorkspaceV2 { .. }
                                | HyprlandEvent::RenameWorkspace { .. }
                                | HyprlandEvent::ActiveSpecialV2 { .. }
                                | HyprlandEvent::MonitorAddedV2 { .. }
                                | HyprlandEvent::MonitorRemovedV2 { .. } => {
                                    let _ = out.send(HyprlandWorkspacesCommandOutput::WorkspacesChanged);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        });
    }

    fn workspaces_with_dividers(mut workspaces: Vec<Arc<Workspace>>) -> Vec<WsRow> {
        // Sort by monitor then id
        workspaces.sort_by_key(|w| (w.monitor.get(), w.id.get()));

        let mut out = Vec::with_capacity(workspaces.len() + 4);
        let mut last_monitor: Option<MonitorId> = None;

        for workspace in workspaces {
            if let Some(monitor) = workspace.monitor_id.get() {
                if last_monitor.is_some_and(|m| m != monitor) {
                    out.push(WsRow::Divider(monitor));
                }

                out.push(WsRow::Workspace(workspace));
                last_monitor = Some(monitor);
            }
        }

        out
    }
}