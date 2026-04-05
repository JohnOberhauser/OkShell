use relm4::{
    gtk,
    ComponentParts,
    ComponentSender,
    SimpleComponent,
};
use relm4::gtk::Orientation;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use okshell_utils::hypr_picker::spawn_color_picker;

#[derive(Debug, Clone)]
pub(crate) struct HyprPickerModel {
    orientation: Orientation,
}

#[derive(Debug)]
pub(crate) enum HyprPickerInput {
    Clicked,
}

#[derive(Debug)]
pub(crate) enum HyprPickerOutput {}

pub(crate) struct HyprPickerInit {
    pub(crate) orientation: Orientation,
}

#[relm4::component(pub)]
impl SimpleComponent for HyprPickerModel {
    type Input = HyprPickerInput;
    type Output = HyprPickerOutput;
    type Init = HyprPickerInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "hypr-picker-bar-widget",
            set_hexpand: model.orientation == Orientation::Vertical,
            set_vexpand: model.orientation == Orientation::Horizontal,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            
            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-bar-widget"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(HyprPickerInput::Clicked);
                },

                #[name="image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("color-select-symbolic"),
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HyprPickerModel {
            orientation: params.orientation,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        _sender: ComponentSender<Self>,
    ) {
        match message {
            HyprPickerInput::Clicked => {
                spawn_color_picker(0)
            }
        }
    }
}
