use crate::common_widgets::revealer_button::revealer_button_icon_label::{
    RevealerButtonIconLabelInit, RevealerButtonIconLabelInput, RevealerButtonIconLabelModel,
};
use okshell_common::WatcherToken;
use okshell_services::power_profile_service;
use okshell_utils::power_profile::{get_power_profile_label, spawn_active_profile_watcher};
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, gtk};
use wayle_power_profiles::types::profile::Profile;

pub(crate) struct ProfileRevealerButtonModel {
    profile: Profile,
    content: Controller<RevealerButtonIconLabelModel>,
    watcher_token: WatcherToken,
}

#[derive(Debug)]
pub(crate) enum ProfileRevealerButtonInput {
    Clicked,
    ActiveProfileChanged,
    Revealed,
    Hidden,
}

#[derive(Debug)]
pub(crate) enum ProfileRevealerButtonOutput {}

pub(crate) struct ProfileRevealerButtonInit {
    pub profile: Profile,
}

#[derive(Debug)]
pub(crate) enum ProfileRevealerButtonCommandOutput {
    ActiveProfileChanged,
}

#[relm4::component(pub)]
impl Component for ProfileRevealerButtonModel {
    type CommandOutput = ProfileRevealerButtonCommandOutput;
    type Input = ProfileRevealerButtonInput;
    type Output = ProfileRevealerButtonOutput;
    type Init = ProfileRevealerButtonInit;

    view! {
        #[root]
        gtk::Box {
            gtk::Button {
                add_css_class: "ok-button-surface",
                set_hexpand: true,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(ProfileRevealerButtonInput::Clicked);
                },

                model.content.widget().clone() {},
            },
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut watcher_token = WatcherToken::new();

        let token = watcher_token.reset();

        spawn_active_profile_watcher(&sender, Some(token), || {
            ProfileRevealerButtonCommandOutput::ActiveProfileChanged
        });

        let button_content = RevealerButtonIconLabelModel::builder()
            .launch(RevealerButtonIconLabelInit {
                label: get_power_profile_label(&params.profile.profile).to_string(),
                icon_name: "".to_string(),
                secondary_icon_name: "".to_string(),
            })
            .detach();

        let model = ProfileRevealerButtonModel {
            profile: params.profile,
            content: button_content,
            watcher_token,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ProfileRevealerButtonInput::Clicked => {
                let profile = self.profile.clone();
                tokio::spawn(async move {
                    let service = power_profile_service();
                    let _ = service
                        .power_profiles
                        .set_active_profile(profile.profile)
                        .await;
                });
            }
            ProfileRevealerButtonInput::ActiveProfileChanged => {
                let active_profile = power_profile_service().power_profiles.active_profile.get();

                if active_profile.eq(&self.profile.profile) {
                    self.content
                        .emit(RevealerButtonIconLabelInput::SetPrimaryIconName(
                            "check-circle-symbolic".to_string(),
                        ))
                } else {
                    self.content
                        .emit(RevealerButtonIconLabelInput::SetPrimaryIconName(
                            "".to_string(),
                        ))
                }
            }
            ProfileRevealerButtonInput::Revealed => {
                let token = self.watcher_token.reset();

                spawn_active_profile_watcher(&sender, Some(token), || {
                    ProfileRevealerButtonCommandOutput::ActiveProfileChanged
                });
            }
            ProfileRevealerButtonInput::Hidden => {
                self.watcher_token.reset();
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ProfileRevealerButtonCommandOutput::ActiveProfileChanged => {
                sender.input(ProfileRevealerButtonInput::ActiveProfileChanged);
            }
        }
    }
}
