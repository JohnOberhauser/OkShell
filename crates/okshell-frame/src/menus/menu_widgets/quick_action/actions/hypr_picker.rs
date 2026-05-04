use okshell_utils::hypr_picker::spawn_color_picker;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};

#[derive(Debug, Clone)]
pub(crate) struct HyprPickerModel {}

#[derive(Debug)]
pub(crate) enum HyprPickerInput {
    Clicked,
}

#[derive(Debug)]
pub(crate) enum HyprPickerOutput {
    CloseMenu,
}

pub(crate) struct HyprPickerInit {}

#[relm4::component(pub)]
impl SimpleComponent for HyprPickerModel {
    type Input = HyprPickerInput;
    type Output = HyprPickerOutput;
    type Init = HyprPickerInit;

    view! {
        #[root]
        gtk::Box {
            #[name = "button"]
            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-button-medium"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(HyprPickerInput::Clicked);
                },

                #[name = "action_icon_image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("color-select-symbolic"),
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HyprPickerModel {};

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            HyprPickerInput::Clicked => {
                let _ = sender.output(HyprPickerOutput::CloseMenu);
                spawn_color_picker(300);
            }
        }
    }
}
