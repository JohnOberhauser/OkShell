use okshell_services::notification_service;
use okshell_utils::notifications::spawn_dnd_watcher;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use relm4::{Component, ComponentParts, ComponentSender, gtk};

#[derive(Debug, Clone)]
pub(crate) struct DoNotDisturbModel {
    enabled: bool,
}

#[derive(Debug)]
pub(crate) enum DoNotDisturbInput {
    Clicked,
}

#[derive(Debug)]
pub(crate) enum DoNotDisturbOutput {}

pub(crate) struct DoNotDisturbInit {}

#[derive(Debug)]
pub(crate) enum DoNotDisturbCommandOutput {
    DndChanged,
}

#[relm4::component(pub)]
impl Component for DoNotDisturbModel {
    type Input = DoNotDisturbInput;
    type Output = DoNotDisturbOutput;
    type Init = DoNotDisturbInit;
    type CommandOutput = DoNotDisturbCommandOutput;

    view! {
        #[root]
        gtk::Box {
            #[name = "button"]
            gtk::Button {
                #[watch]
                set_css_classes: if model.enabled {
                    &["ok-button-surface", "ok-button-medium", "selected"]
                } else {
                    &["ok-button-surface", "ok-button-medium"]
                },
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(DoNotDisturbInput::Clicked);
                },

                #[name = "action_icon_image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    #[watch]
                    set_icon_name: if model.enabled {
                        Some("notification-disabled-symbolic")
                    } else {
                        Some("notification-symbolic")
                    },
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_dnd_watcher(&sender, || DoNotDisturbCommandOutput::DndChanged);

        let model = DoNotDisturbModel { enabled: false };

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
            DoNotDisturbInput::Clicked => {
                let service = notification_service();
                let dnd = service.dnd.get();

                service.set_dnd(!dnd);
            }
        }

        self.update_view(widgets, sender);
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            DoNotDisturbCommandOutput::DndChanged => {
                let service = notification_service();
                self.enabled = service.dnd.get();
            }
        }

        self.update_view(widgets, sender);
    }
}
