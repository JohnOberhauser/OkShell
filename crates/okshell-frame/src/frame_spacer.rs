use crate::bars::bar::BarType;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{
    BarsStoreFields, ConfigStoreFields, HorizontalBarStoreFields, VerticalBarStoreFields,
};
use reactive_graph::traits::GetUntracked;
use relm4::gtk::Orientation;
use relm4::gtk::gdk::Monitor;
use relm4::gtk::prelude::{GtkWindowExt, WidgetExt};
use relm4::{Component, ComponentParts, ComponentSender, gtk};

#[derive(Debug, Clone)]
pub(crate) struct FrameSpacerModel {
    orientation: Orientation,
    width: i32,
    height: i32,
    border_width: i32,
    border_height: i32,
    revealed: bool,
}

#[derive(Debug)]
pub(crate) enum FrameSpacerInput {
    WidthUpdated(i32),
    HeightUpdated(i32),
    BorderWidthUpdated(i32),
    BorderHeightUpdated(i32),
    SetRevealed(bool),
    ToggleRevealed,
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

            gtk::Revealer {
                #[watch]
                set_reveal_child: model.revealed,
                set_transition_type: transition_type,

                #[name = "spacer"]
                gtk::Box {
                    #[watch]
                    set_width_request: model.width + model.border_width,
                    #[watch]
                    set_height_request: model.height + model.border_height,
                },
            },
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
        root.set_namespace(Some("okshell-spacer"));

        let transition_type: gtk::RevealerTransitionType;
        let reveal_by_default: bool;

        match params.bar_type {
            BarType::Top => {
                root.set_anchor(Edge::Top, true);
                root.set_anchor(Edge::Left, true);
                root.set_anchor(Edge::Right, true);
                transition_type = gtk::RevealerTransitionType::SlideDown;
                reveal_by_default = config_manager()
                    .config()
                    .bars()
                    .top_bar()
                    .reveal_by_default()
                    .get_untracked();
            }
            BarType::Bottom => {
                root.set_anchor(Edge::Bottom, true);
                root.set_anchor(Edge::Left, true);
                root.set_anchor(Edge::Right, true);
                transition_type = gtk::RevealerTransitionType::SlideUp;
                reveal_by_default = config_manager()
                    .config()
                    .bars()
                    .bottom_bar()
                    .reveal_by_default()
                    .get_untracked();
            }
            BarType::Left => {
                root.set_anchor(Edge::Top, true);
                root.set_anchor(Edge::Bottom, true);
                root.set_anchor(Edge::Left, true);
                transition_type = gtk::RevealerTransitionType::SlideRight;
                reveal_by_default = config_manager()
                    .config()
                    .bars()
                    .left_bar()
                    .reveal_by_default()
                    .get_untracked();
            }
            BarType::Right => {
                root.set_anchor(Edge::Top, true);
                root.set_anchor(Edge::Bottom, true);
                root.set_anchor(Edge::Right, true);
                transition_type = gtk::RevealerTransitionType::SlideLeft;
                reveal_by_default = config_manager()
                    .config()
                    .bars()
                    .right_bar()
                    .reveal_by_default()
                    .get_untracked();
            }
        }

        let orientation = match params.bar_type {
            BarType::Top => Orientation::Horizontal,
            BarType::Bottom => Orientation::Horizontal,
            BarType::Left => Orientation::Vertical,
            BarType::Right => Orientation::Vertical,
        };
        let model = FrameSpacerModel {
            orientation,
            width: 0,
            height: 0,
            border_width: 0,
            border_height: 0,
            revealed: reveal_by_default,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            FrameSpacerInput::WidthUpdated(val) => {
                self.width = val;
            }
            FrameSpacerInput::HeightUpdated(val) => {
                self.height = val;
            }
            FrameSpacerInput::BorderWidthUpdated(width) => self.border_width = width,
            FrameSpacerInput::BorderHeightUpdated(height) => {
                self.border_height = height;
            }
            FrameSpacerInput::SetRevealed(revealed) => {
                self.revealed = revealed;
            }
            FrameSpacerInput::ToggleRevealed => {
                self.revealed = !self.revealed;
            }
        }
        self.update_view(widgets, sender);
    }
}
