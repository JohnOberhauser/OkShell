use relm4::gtk::glib;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};

pub(crate) struct RevealerRowSliderModel {
    value_changed_signal: glib::SignalHandlerId,
}

#[derive(Debug)]
pub(crate) enum RevealerRowSliderInput {
    SetValue(f64),
}

#[derive(Debug)]
pub(crate) enum RevealerRowSliderOutput {
    ValueChanged(f64),
}

pub(crate) struct RevealerRowSliderInit {}

#[derive(Debug)]
pub(crate) enum RevealerRowSliderCommandOutput {}

#[relm4::component(pub)]
impl Component for RevealerRowSliderModel {
    type CommandOutput = RevealerRowSliderCommandOutput;
    type Input = RevealerRowSliderInput;
    type Output = RevealerRowSliderOutput;
    type Init = RevealerRowSliderInit;

    view! {
        #[root]
        #[name = "scale"]
        gtk::Scale {
            add_css_class: "ok-progress-bar",
            set_hexpand: true,
            set_can_focus: false,
            set_focus_on_click: false,
            set_range: (0.0, 1.0),
            set_margin_end: 20,
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();

        let signal_id = widgets.scale.connect_value_changed(move |scale| {
            let _ = sender.output(RevealerRowSliderOutput::ValueChanged(scale.value()));
        });

        let model = RevealerRowSliderModel {
            value_changed_signal: signal_id,
        };

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            RevealerRowSliderInput::SetValue(value) => {
                widgets.scale.block_signal(&self.value_changed_signal);
                widgets.scale.set_value(value);
                widgets.scale.unblock_signal(&self.value_changed_signal);
            }
        }
    }
}
