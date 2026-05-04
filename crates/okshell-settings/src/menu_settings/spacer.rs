use okshell_config::schema::menu_widgets::SpacerConfig;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};

#[derive(Debug)]
pub struct SpacerConfigModel {
    size: i32,
}

#[derive(Debug)]
pub enum SpacerConfigInput {
    SizeChanged(i32),
}

#[derive(Debug)]
pub enum SpacerConfigOutput {
    SizeChanged(i32),
}

#[relm4::component(pub)]
impl Component for SpacerConfigModel {
    type CommandOutput = ();
    type Input = SpacerConfigInput;
    type Output = SpacerConfigOutput;
    type Init = SpacerConfig;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 8,

            gtk::Label {
                add_css_class: "label-small",
                set_label: "Size",
                set_halign: gtk::Align::Start,
                set_hexpand: true,
            },

            gtk::SpinButton {
                set_range: (0.0, 500.0),
                set_increments: (1.0, 10.0),
                #[watch]
                set_value: model.size as f64,
                connect_value_changed[sender] => move |s| {
                    sender.input(SpacerConfigInput::SizeChanged(s.value() as i32));
                },
            },
        }
    }

    fn init(
        config: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SpacerConfigModel { size: config.size };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            SpacerConfigInput::SizeChanged(s) => {
                self.size = s;
                let _ = sender.output(SpacerConfigOutput::SizeChanged(s));
            }
        }
    }
}
