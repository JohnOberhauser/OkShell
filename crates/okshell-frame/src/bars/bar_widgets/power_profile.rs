use okshell_utils::power_profile::{get_active_power_profile_icon, spawn_active_profile_watcher};
use relm4::gtk::prelude::WidgetExt;
use relm4::{Component, ComponentParts, ComponentSender, gtk};

#[derive(Debug, Clone)]
pub(crate) struct PowerProfileModel {}

#[derive(Debug)]
pub(crate) enum PowerProfileInput {}

#[derive(Debug)]
pub(crate) enum PowerProfileOutput {}

pub(crate) struct PowerProfileInit {}

#[derive(Debug)]
pub(crate) enum PowerProfileCommandOutput {
    ProfileChanged,
}

#[relm4::component(pub)]
impl Component for PowerProfileModel {
    type CommandOutput = PowerProfileCommandOutput;
    type Input = PowerProfileInput;
    type Output = PowerProfileOutput;
    type Init = PowerProfileInit;

    view! {
        #[root]
        gtk::Box {
            set_css_classes: &["ok-button-surface", "ok-bar-widget", "power-profile-bar-widget"],
            set_hexpand: false,
            set_vexpand: false,

            #[name="image"]
            gtk::Image {
                set_hexpand: true,
                set_vexpand: true,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
            }
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_active_profile_watcher(&sender, None, || PowerProfileCommandOutput::ProfileChanged);

        let model = PowerProfileModel {};

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            PowerProfileCommandOutput::ProfileChanged => {
                widgets
                    .image
                    .set_icon_name(Some(get_active_power_profile_icon()));
            }
        }
    }
}
