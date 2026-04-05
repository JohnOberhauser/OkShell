use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::gdk::Monitor;
use relm4::gtk::Orientation;
use relm4::gtk::prelude::{GtkWindowExt, WidgetExt};
use crate::bars::bar::BarType;

#[derive(Debug, Clone)]
pub(crate) struct FrameSpacerModel {
    orientation: Orientation,
}

#[derive(Debug)]
pub(crate) enum FrameSpacerInput {
    WidthUpdated(i32),
    HeightUpdated(i32),
}

#[derive(Debug)]
pub(crate) enum FrameSpacerOutput {}

pub(crate) struct FrameSpacerInit {
    pub bar_type: BarType,
    pub monitor: Monitor,
}

#[relm4::component(pub)]
impl Component for FrameSpacerModel {
    type CommandOutput = ();
    type Input = FrameSpacerInput;
    type Output = FrameSpacerOutput;
    type Init = FrameSpacerInit;

    view! {
        #[root]
        gtk::Window {
            add_css_class: "frame-spacer-window",
            set_default_height: if model.orientation == Orientation::Horizontal {
                1
            } else {
                0
            },
            set_default_width: if model.orientation == Orientation::Vertical {
                1
            } else {
                0
            },
            set_can_target: false,
            set_can_focus: false,

            #[name = "spacer"]
            gtk::Box {

            }
        },
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        root.init_layer_shell();
        root.set_monitor(Some(&params.monitor));
        root.set_layer(Layer::Background);
        root.auto_exclusive_zone_enable();
        root.set_decorated(false);
        root.set_visible(true);

        match params.bar_type {
            BarType::Top => {
                root.set_anchor(Edge::Top, true);
                root.set_anchor(Edge::Left, true);
                root.set_anchor(Edge::Right, true);
            }
            BarType::Bottom => {
                root.set_anchor(Edge::Bottom, true);
                root.set_anchor(Edge::Left, true);
                root.set_anchor(Edge::Right, true);
            }
            BarType::Left => {
                root.set_anchor(Edge::Top, true);
                root.set_anchor(Edge::Bottom, true);
                root.set_anchor(Edge::Left, true);
            }
            BarType::Right => {
                root.set_anchor(Edge::Top, true);
                root.set_anchor(Edge::Bottom, true);
                root.set_anchor(Edge::Right, true);
            }
        }

        let orientation = match params.bar_type {
            BarType::Top => {
                Orientation::Horizontal
            }
            BarType::Bottom => {
                Orientation::Horizontal
            }
            BarType::Left => {
                Orientation::Vertical
            }
            BarType::Right => {
                Orientation::Vertical
            }
        };
        let model = FrameSpacerModel {
            orientation
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            FrameSpacerInput::WidthUpdated(val) => {
                widgets.spacer.set_width_request(val);
            }
            FrameSpacerInput::HeightUpdated(val) => {
                widgets.spacer.set_height_request(val);
            }
        }
        self.update_view(widgets, sender);
    }
}