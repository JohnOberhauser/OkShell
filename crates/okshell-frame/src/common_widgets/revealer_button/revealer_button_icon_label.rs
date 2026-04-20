use std::ops::Not;
use relm4::prelude::*;
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::gtk::pango;

pub struct RevealerButtonIconLabelModel {
    pub label: String,
    pub icon_name: String,
    pub secondary_icon_name: String,
}

#[derive(Debug)]
pub enum RevealerButtonIconLabelInput {
    SetPrimaryIconName(String),
    SetSecondaryIconName(String),
}

pub struct RevealerButtonIconLabelInit {
    pub label: String,
    pub icon_name: String,
    pub secondary_icon_name: String,
}

#[relm4::component(pub)]
impl SimpleComponent for RevealerButtonIconLabelModel {
    type Init = RevealerButtonIconLabelInit;
    type Input = RevealerButtonIconLabelInput;
    type Output = ();

    view! {
        gtk::Box{
            #[name = "image"]
            gtk::Image {
                add_css_class: "revealer-button-icon-label-icon",
                set_margin_end: 12,
                #[watch]
                set_icon_name: Some(model.icon_name.as_str()),
            },

            #[name = "label"]
            gtk::Label {
                add_css_class: "label-small",
                set_halign: gtk::Align::Start,
                set_hexpand: true,
                set_ellipsize: pango::EllipsizeMode::End,
                #[watch]
                set_label: model.label.as_str(),
            },

            #[name = "secondary_image"]
            gtk::Image {
                #[watch]
                set_visible: model.secondary_icon_name.is_empty().not(),
                add_css_class: "revealer-button-icon-label-icon",
                set_margin_start: 12,
                #[watch]
                set_icon_name: Some(model.secondary_icon_name.as_str()),
            },
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>
    ) -> ComponentParts<Self> {
        let model = RevealerButtonIconLabelModel {
            label: params.label,
            icon_name: params.icon_name,
            secondary_icon_name: params.secondary_icon_name,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        _sender: ComponentSender<Self>
    ) {
        match msg {
            RevealerButtonIconLabelInput::SetPrimaryIconName(icon_name) => {
                self.icon_name = icon_name;
            }
            RevealerButtonIconLabelInput::SetSecondaryIconName(icon_name) => {
                self.secondary_icon_name = icon_name;
            }
        }
    }
}