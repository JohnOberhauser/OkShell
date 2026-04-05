use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use okshell_settings::open_settings;

pub(crate) struct SettingsModel {}

#[derive(Debug)]
pub(crate) enum SettingsInput {
    Clicked,
}

#[derive(Debug)]
pub(crate) enum SettingsOutput {
    CloseMenu,
}

pub(crate) struct SettingsInit {
}

#[relm4::component(pub)]
impl SimpleComponent for SettingsModel {
    type Input = SettingsInput;
    type Output = SettingsOutput;
    type Init = SettingsInit;

    view! {
        #[root]
        gtk::Box {
            #[name = "button"]
            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-button-medium"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(SettingsInput::Clicked);
                },

                #[name = "action_icon_image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("settings-symbolic"),
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SettingsModel {};

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        sender: ComponentSender<Self>,
    ) {
        match message {
            SettingsInput::Clicked => {
                let _ = sender.output(SettingsOutput::CloseMenu);
                open_settings();
            }
        }
    }
}
