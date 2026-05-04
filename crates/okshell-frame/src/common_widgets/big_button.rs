use relm4::gtk::{self, prelude::*};
use relm4::prelude::*;

#[relm4::widget_template(pub)]
impl WidgetTemplate for BigButton {
    view! {
        gtk::Button {
            set_css_classes: &["ok-button-primary", "ok-button-large"],

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
                set_spacing: 4,

                #[name = "icon"]
                gtk::Image {
                },

                #[name = "label"]
                gtk::Label {
                    add_css_class: "label-small-primary",
                },
            }
        }
    }
}
