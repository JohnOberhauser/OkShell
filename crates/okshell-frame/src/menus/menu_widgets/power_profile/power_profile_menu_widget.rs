use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use relm4::gtk::prelude::WidgetExt;
use wayle_power_profiles::types::profile::{PowerProfile};
use okshell_services::{power_profile_service};
use crate::common_widgets::revealer_row::revealer_row::{RevealerRowInit, RevealerRowInput, RevealerRowModel, RevealerRowOutput};
use crate::common_widgets::revealer_row::revealer_row_label::{RevealerRowLabelInit, RevealerRowLabelInput, RevealerRowLabelModel};
use crate::menus::menu_widgets::power_profile::power_profile_revealed_content::{PowerProfileRevealedContentInit, PowerProfileRevealedContentInput, PowerProfileRevealedContentModel};
use okshell_utils::power_profile::{get_power_profile_icon, get_power_profile_label, spawn_active_profile_watcher};

pub(crate) struct PowerProfileMenuWidgetModel {
    revealer_row: Controller<RevealerRowModel<RevealerRowLabelModel, PowerProfileRevealedContentModel>>,
}

#[derive(Debug)]
pub(crate) enum PowerProfileMenuWidgetInput {
    ActionButtonClicked,
    RevealerRowRevealed,
    RevealerRowHidden,
    ParentRevealChanged(bool),
    UpdateProfile(PowerProfile),
}

#[derive(Debug)]
pub(crate) enum PowerProfileMenuWidgetOutput {}

pub(crate) struct PowerProfileMenuWidgetInit {}

#[derive(Debug)]
pub(crate) enum PowerProfileMenuWidgetCommandOutput {
    ProfileChanged,
}

#[relm4::component(pub)]
impl Component for PowerProfileMenuWidgetModel {
    type CommandOutput = PowerProfileMenuWidgetCommandOutput;
    type Input = PowerProfileMenuWidgetInput;
    type Output = PowerProfileMenuWidgetOutput;
    type Init = PowerProfileMenuWidgetInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "power-profiles-menu-widget",
            
            model.revealer_row.widget().clone() {},
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        spawn_active_profile_watcher(
            &sender,
            None,
            || PowerProfileMenuWidgetCommandOutput::ProfileChanged,
        );

        let row_content = RevealerRowLabelModel::builder()
            .launch(RevealerRowLabelInit {
                label: "Power Profile".to_string(),
            })
            .detach();

        let revealed_content = PowerProfileRevealedContentModel::builder()
            .launch(PowerProfileRevealedContentInit{})
            .detach();

        let revealer_row = RevealerRowModel::<RevealerRowLabelModel, PowerProfileRevealedContentModel>::builder()
            .launch(RevealerRowInit {
                icon_name: "power-profile-balanced-symbolic".into(),
                action_button_sensitive: false,
                content: row_content,
                revealed_content,
            })
            .forward(sender.input_sender(), |msg| {
                match msg {
                    RevealerRowOutput::ActionButtonClicked => {
                        PowerProfileMenuWidgetInput::ActionButtonClicked
                    }
                    RevealerRowOutput::Revealed => {
                        PowerProfileMenuWidgetInput::RevealerRowRevealed
                    }
                    RevealerRowOutput::Hidden => {
                        PowerProfileMenuWidgetInput::RevealerRowHidden
                    }
                }
            });

        let model = PowerProfileMenuWidgetModel {
            revealer_row,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            PowerProfileMenuWidgetInput::ActionButtonClicked => {

            }
            PowerProfileMenuWidgetInput::RevealerRowRevealed => {
                self.revealer_row.model().revealed_content.emit(PowerProfileRevealedContentInput::Revealed);
            }
            PowerProfileMenuWidgetInput::RevealerRowHidden => {
                self.revealer_row.model().revealed_content.emit(PowerProfileRevealedContentInput::Hidden);
            }
            PowerProfileMenuWidgetInput::ParentRevealChanged(revealed) => {
                if !revealed {
                    self.revealer_row.emit(RevealerRowInput::SetRevealed(false));
                }
            }
            PowerProfileMenuWidgetInput::UpdateProfile(profile) => {
                self.revealer_row.emit(RevealerRowInput::UpdateActionIconName(
                    get_power_profile_icon(&profile).to_string()
                ));
                self.revealer_row.model().content.emit(RevealerRowLabelInput::SetLabel(
                    format!("Power Profile: {}", get_power_profile_label(&profile).to_string())
                ))
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            PowerProfileMenuWidgetCommandOutput::ProfileChanged => {
                let profile = power_profile_service().power_profiles.active_profile.get();
                sender.input(PowerProfileMenuWidgetInput::UpdateProfile(profile));
            }
        }
    }
}