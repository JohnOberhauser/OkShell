use relm4::prelude::*;
use relm4::gtk;
use relm4::gtk::prelude::*;
use relm4::gtk::pango;

pub struct RevealerRowLabelModel {
    label: String,
}

#[derive(Debug)]
pub enum RevealerRowLabelInput {
    SetLabel(String),
}

pub struct RevealerRowLabelInit {
    pub label: String,
}

#[relm4::component(pub)]
impl SimpleComponent for RevealerRowLabelModel {
    type Init = RevealerRowLabelInit;
    type Input = RevealerRowLabelInput;
    type Output = ();

    view! {
        #[name = "label"]
        gtk::Label {
            add_css_class: "label-medium-bold",
            set_halign: gtk::Align::Start,
            set_hexpand: true,
            set_ellipsize: pango::EllipsizeMode::End,
            #[watch]
            set_label: model.label.as_str(),
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>
    ) -> ComponentParts<Self> {
        let model = RevealerRowLabelModel {
            label: params.label,
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
            RevealerRowLabelInput::SetLabel(label) => {
                self.label = label;
            },
        }
    }
}