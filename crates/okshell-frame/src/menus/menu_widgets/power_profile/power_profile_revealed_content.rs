use crate::menus::menu_widgets::power_profile::profile_revealer_button::{
    ProfileRevealerButtonInit, ProfileRevealerButtonInput, ProfileRevealerButtonModel,
};
use okshell_common::WatcherToken;
use okshell_common::dynamic_box::dynamic_box::{
    DynamicBoxFactory, DynamicBoxInit, DynamicBoxInput, DynamicBoxModel,
};
use okshell_common::dynamic_box::generic_widget_controller::{
    GenericWidgetController, GenericWidgetControllerExtSafe,
};
use okshell_services::power_profile_service;
use okshell_utils::power_profile::spawn_profiles_watcher;
use relm4::gtk::RevealerTransitionType;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, gtk};
use wayle_power_profiles::types::profile::Profile;

pub(crate) struct PowerProfileRevealedContentModel {
    dynamic_box_controller: Controller<DynamicBoxModel<Profile, String>>,
    watcher_token: WatcherToken,
}

#[derive(Debug)]
pub(crate) enum PowerProfileRevealedContentInput {
    UpdateProfiles,
    Revealed,
    Hidden,
}

#[derive(Debug)]
pub(crate) enum PowerProfileRevealedContentOutput {}

pub(crate) struct PowerProfileRevealedContentInit {}

#[derive(Debug)]
pub(crate) enum PowerProfileRevealedContentCommandOutput {
    ProfilesUpdated,
}

#[relm4::component(pub)]
impl Component for PowerProfileRevealedContentModel {
    type CommandOutput = PowerProfileRevealedContentCommandOutput;
    type Input = PowerProfileRevealedContentInput;
    type Output = PowerProfileRevealedContentOutput;
    type Init = PowerProfileRevealedContentInit;

    view! {
        #[root]
        gtk::Box {
            model.dynamic_box_controller.widget().clone() {},
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut watcher_token = WatcherToken::new();

        let token = watcher_token.reset();

        spawn_profiles_watcher(&sender, Some(token), || {
            PowerProfileRevealedContentCommandOutput::ProfilesUpdated
        });

        let devices_dynamic_box_factory = DynamicBoxFactory::<Profile, String> {
            id: Box::new(|item| item.profile.to_string()),
            create: Box::new(move |item| {
                let profile = item.clone();
                let revealer_button = ProfileRevealerButtonModel::builder()
                    .launch(ProfileRevealerButtonInit { profile })
                    .detach();

                Box::new(revealer_button) as Box<dyn GenericWidgetController>
            }),
            update: None,
        };

        let dynamic_box_controller: Controller<DynamicBoxModel<Profile, String>> =
            DynamicBoxModel::builder()
                .launch(DynamicBoxInit {
                    factory: devices_dynamic_box_factory,
                    orientation: gtk::Orientation::Vertical,
                    spacing: 0,
                    transition_type: RevealerTransitionType::SlideDown,
                    transition_duration_ms: 200,
                    reverse: false,
                    retain_entries: false,
                    allow_drag_and_drop: false,
                })
                .detach();

        let model = PowerProfileRevealedContentModel {
            dynamic_box_controller,
            watcher_token: WatcherToken::new(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            PowerProfileRevealedContentInput::UpdateProfiles => {
                let profiles = power_profile_service().power_profiles.profiles.get();
                self.dynamic_box_controller
                    .emit(DynamicBoxInput::SetItems(profiles))
            }
            PowerProfileRevealedContentInput::Revealed => {
                let token = self.watcher_token.reset();

                spawn_profiles_watcher(&sender, Some(token), || {
                    PowerProfileRevealedContentCommandOutput::ProfilesUpdated
                });

                self.dynamic_box_controller
                    .model()
                    .for_each_entry(|_, entry| {
                        if let Some(ctrl) = entry
                            .controller
                            .as_ref()
                            .downcast_ref::<Controller<ProfileRevealerButtonModel>>()
                        {
                            ctrl.emit(ProfileRevealerButtonInput::Revealed);
                        }
                    });
            }
            PowerProfileRevealedContentInput::Hidden => {
                self.watcher_token.reset();

                self.dynamic_box_controller
                    .model()
                    .for_each_entry(|_, entry| {
                        if let Some(ctrl) = entry
                            .controller
                            .as_ref()
                            .downcast_ref::<Controller<ProfileRevealerButtonModel>>()
                        {
                            ctrl.emit(ProfileRevealerButtonInput::Hidden);
                        }
                    });
            }
        }

        self.update_view(widgets, sender);
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            PowerProfileRevealedContentCommandOutput::ProfilesUpdated => {
                sender.input(PowerProfileRevealedContentInput::UpdateProfiles);
            }
        }
    }
}
