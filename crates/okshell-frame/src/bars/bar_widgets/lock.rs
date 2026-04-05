use relm4::{
    gtk,
    ComponentParts,
    ComponentSender,
    SimpleComponent,
};
use relm4::gtk::Orientation;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use okshell_session::session_lock::session_lock;

#[derive(Debug, Clone)]
pub(crate) struct LockModel {
    orientation: Orientation,
}

#[derive(Debug)]
pub(crate) enum LockInput {
    Clicked,
}

#[derive(Debug)]
pub(crate) enum LockOutput {
    CloseMenu,
}

pub(crate) struct LockInit {
    pub(crate) orientation: Orientation,
}

#[relm4::component(pub)]
impl SimpleComponent for LockModel {
    type Input = LockInput;
    type Output = LockOutput;
    type Init = LockInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "lock-bar-widget",
            set_hexpand: model.orientation == Orientation::Vertical,
            set_vexpand: model.orientation == Orientation::Horizontal,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            
            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-bar-widget"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(LockInput::Clicked);
                },

                #[name="image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("system-lock-screen-symbolic"),
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = LockModel {
            orientation: params.orientation,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        sender: ComponentSender<Self>,
    ) {
        match message {
            LockInput::Clicked => {
                let _ = sender.output(LockOutput::CloseMenu);
                session_lock().lock();
            }
        }
    }
}