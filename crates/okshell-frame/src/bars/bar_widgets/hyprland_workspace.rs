use okshell_config::schema::bar_widgets::HyprlandWorkspacesConfig;
use okshell_services::hyprland_service;
use okshell_utils::hyprland::is_an_active_workspace;
use relm4::gtk::pango;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use relm4::{Component, ComponentParts, ComponentSender, gtk};
use std::sync::Arc;
use tracing::error;
use wayle_hyprland::{Workspace, WorkspaceInfo};

#[derive(Debug, Clone)]
pub(crate) struct HyprlandWorkspaceModel {
    workspace: Arc<Workspace>,
    is_active: bool,
    /// `Some(font)` -> show the workspace number in that font; `None` -> show the icon.
    number_font: Option<String>,
    use_letters: bool,
}

#[derive(Debug)]
pub(crate) enum HyprlandWorkspaceInput {
    ActiveUpdate(Vec<WorkspaceInfo>),
    WorkspaceClicked,
}

pub(crate) struct HyprlandWorkspaceInit {
    pub(crate) workspace: Arc<Workspace>,
    pub(crate) config: HyprlandWorkspacesConfig,
}

#[derive(Debug)]
pub(crate) enum HyprlandWorkspaceOutput {}

#[relm4::component(pub)]
impl Component for HyprlandWorkspaceModel {
    type CommandOutput = ();
    type Input = HyprlandWorkspaceInput;
    type Output = HyprlandWorkspaceOutput;
    type Init = HyprlandWorkspaceInit;

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

                gtk::Box {
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,

                    #[name="image"]
                    gtk::Image {
                        set_visible: model.number_font.is_none(),
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        set_icon_name: Some("workspace-symbolic"),
                    },

                    #[name="label"]
                    gtk::Label {
                        add_css_class: "label-large-bold",
                        set_visible: model.number_font.is_some(),
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        set_label: &workspace_label(&model.workspace.address.get().to_string(), model.use_letters),
                    },
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let is_active = is_an_active_workspace(&params.workspace);

        let font = params.config.font.trim();
        let number_font = (!font.is_empty()).then(|| font.to_string());

        let model = HyprlandWorkspaceModel {
            workspace: params.workspace,
            is_active,
            number_font,
            use_letters: params.config.use_letters,
        };

        let widgets = view_output!();

        if let Some(font) = &model.number_font {
            widgets.label.set_attributes(Some(&font_attrs(font)));
        }
        model.apply_active(&widgets);

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
                let address = self.workspace.address.get();
                self.is_active = workspace_infos.iter().any(|p| p.address == address);
                self.apply_active(widgets);
            }
            HyprlandWorkspaceInput::WorkspaceClicked => {
                let hyprland = hyprland_service();
                let workspace_address = self.workspace.address.get();
                tokio::spawn(async move {
                    let command =
                        format!("hl.dsp.focus({{ workspace = \"{}\" }})", workspace_address);
                    if let Err(e) = hyprland.dispatch(&command).await {
                        error!(error = %e, workspace = workspace_address.to_string(), "Failed to switch workspace");
                    }
                });
            }
        }
    }
}

impl HyprlandWorkspaceModel {
    fn apply_active(&self, widgets: &HyprlandWorkspaceModelWidgets) {
        if self.number_font.is_some() {
            if self.is_active {
                widgets
                    .label
                    .add_css_class("hyprland-workspace-text-active");
            } else {
                widgets
                    .label
                    .remove_css_class("hyprland-workspace-text-active");
            }
        } else {
            let icon = if self.is_active {
                "workspace-selected-symbolic"
            } else {
                "workspace-symbolic"
            };
            widgets.image.set_icon_name(Some(icon));
        }
    }
}

/// Parses a Pango font string like "JetBrains Mono Bold 11" or just
/// "JetBrains Mono". Fields left out (e.g. size) keep the label's CSS value.
fn font_attrs(font: &str) -> pango::AttrList {
    let attrs = pango::AttrList::new();
    let desc = pango::FontDescription::from_string(font);
    attrs.insert(pango::AttrFontDesc::new(&desc));
    attrs
}

fn workspace_label(address: &str, use_letters: bool) -> String {
    match address.parse::<u32>() {
        Ok(n @ 1..=26) if use_letters => char::from(b'a' + (n - 1) as u8).to_string(),
        _ => address.to_string(),
    }
}
