use okshell_session::session_lock::session_lock;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};

#[derive(Debug, Clone)]
pub(crate) struct LockModel {}

#[derive(Debug)]
pub(crate) enum LockInput {
    Clicked,
}

#[derive(Debug)]
pub(crate) enum LockOutput {
    CloseMenu,
}

pub(crate) struct LockInit {}

#[relm4::component(pub)]
impl SimpleComponent for LockModel {
    type Input = LockInput;
    type Output = LockOutput;
    type Init = LockInit;

    view! {
        #[root]
        gtk::Box {
            #[name = "button"]
            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-button-medium"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(LockInput::Clicked);
                },

                #[name = "action_icon_image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("system-lock-screen-symbolic"),
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = LockModel {};

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            LockInput::Clicked => {
                let _ = sender.output(LockOutput::CloseMenu);
                session_lock().lock();
            }
        }
    }
}
