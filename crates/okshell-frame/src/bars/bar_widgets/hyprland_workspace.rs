use okshell_services::hyprland_service;
use okshell_utils::hyprland::is_an_active_workspace;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use relm4::{Component, ComponentParts, ComponentSender, gtk};
use std::sync::Arc;
use tracing::error;
use wayle_hyprland::{Workspace, WorkspaceInfo};

#[derive(Debug, Clone)]
pub(crate) struct HyprlandWorkspaceModel {
    workspace: Arc<Workspace>,
    is_active: bool,
}

#[derive(Debug)]
pub(crate) enum HyprlandWorkspaceInput {
    ActiveUpdate(Vec<WorkspaceInfo>),
    WorkspaceClicked,
}

#[derive(Debug)]
pub(crate) enum HyprlandWorkspaceOutput {}

#[relm4::component(pub)]
impl Component for HyprlandWorkspaceModel {
    type CommandOutput = ();
    type Input = HyprlandWorkspaceInput;
    type Output = HyprlandWorkspaceOutput;
    type Init = Arc<Workspace>;

    view! {
        #[root]
        gtk::Box {
            set_hexpand: false,
            set_vexpand: false,

            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-bar-widget"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(HyprlandWorkspaceInput::WorkspaceClicked);
                },

                #[name="image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("workspace-symbolic"),
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let is_active = is_an_active_workspace(&params);

        let model = HyprlandWorkspaceModel {
            workspace: params,
            is_active,
        };

        let widgets = view_output!();

        if model.is_active {
            widgets
                .image
                .set_icon_name(Some("workspace-selected-symbolic"));
        }

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            HyprlandWorkspaceInput::ActiveUpdate(workspace_infos) => {
                self.is_active = workspace_infos
                    .iter()
                    .find(|p| p.id == self.workspace.id.get())
                    .is_some();
                if self.is_active {
                    widgets
                        .image
                        .set_icon_name(Some("workspace-selected-symbolic"));
                } else {
                    widgets.image.set_icon_name(Some("workspace-symbolic"));
                }
            }
            HyprlandWorkspaceInput::WorkspaceClicked => {
                let hyprland = hyprland_service();
                let workspace_id = self.workspace.id.get();
                tokio::spawn(async move {
                    let command = format!("hl.dsp.focus({{ workspace = \"{}\" }})", workspace_id);
                    if let Err(e) = hyprland.dispatch(&command).await {
                        error!(error = %e, workspace = workspace_id, "Failed to switch workspace");
                    }
                });
            }
        }
    }
}
