use okshell_idle::inhibitor::IdleInhibitor;
use okshell_utils::idle::spawn_idle_inhibitor_watcher;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use relm4::{Component, ComponentParts, ComponentSender, gtk};

#[derive(Debug, Clone)]
pub(crate) struct IdleInhibitorModel {
    enabled: bool,
}

#[derive(Debug)]
pub(crate) enum IdleInhibitorInput {
    Clicked,
}

#[derive(Debug)]
pub(crate) enum IdleInhibitorOutput {}

pub(crate) struct IdleInhibitorInit {}

#[derive(Debug)]
pub(crate) enum IdleInhibitorCommandOutput {
    InhibitorStateChanged,
}

#[relm4::component(pub)]
impl Component for IdleInhibitorModel {
    type CommandOutput = IdleInhibitorCommandOutput;
    type Input = IdleInhibitorInput;
    type Output = IdleInhibitorOutput;
    type Init = IdleInhibitorInit;

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
                    sender.input(IdleInhibitorInput::Clicked);
                },

                #[name = "action_icon_image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("coffee-symbolic"),
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_idle_inhibitor_watcher(&sender, || {
            IdleInhibitorCommandOutput::InhibitorStateChanged
        });

        let inhibitor = IdleInhibitor::global();

        let model = IdleInhibitorModel {
            enabled: inhibitor.get(),
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
            IdleInhibitorInput::Clicked => {
                tokio::spawn(async move {
                    let inhibitor = IdleInhibitor::global();
                    let _ = inhibitor.toggle().await;
                });
            }
        }

        self.update_view(widgets, sender);
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            IdleInhibitorCommandOutput::InhibitorStateChanged => {
                let inhibitor = IdleInhibitor::global();
                self.enabled = inhibitor.get();
            }
        }
    }
}
