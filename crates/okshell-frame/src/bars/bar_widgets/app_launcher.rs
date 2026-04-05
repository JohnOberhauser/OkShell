use relm4::{
    gtk,
    ComponentParts,
    ComponentSender,
    SimpleComponent,
};
use relm4::gtk::Orientation;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};

#[derive(Debug, Clone)]
pub(crate) struct AppLauncherModel {
    orientation: Orientation,
}

#[derive(Debug)]
pub(crate) enum AppLauncherInput {}

#[derive(Debug)]
pub(crate) enum AppLauncherOutput {
    Clicked,
}

pub(crate) struct AppLauncherInit {
    pub orientation: Orientation,
}

#[relm4::component(pub)]
impl SimpleComponent for AppLauncherModel {
    type Input = AppLauncherInput;
    type Output = AppLauncherOutput;
    type Init = AppLauncherInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "app-launcher-bar-widget",
            set_hexpand: model.orientation == Orientation::Vertical,
            set_vexpand: model.orientation == Orientation::Horizontal,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,

            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-bar-widget"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.output(AppLauncherOutput::Clicked).unwrap_or_default();
                },

                #[name="image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("view-app-grid-symbolic"),
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppLauncherModel {
            orientation: params.orientation,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}