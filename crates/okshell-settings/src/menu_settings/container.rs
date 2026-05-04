use crate::menu_settings::menu_widget_list::{
    MenuWidgetListInit, MenuWidgetListModel, MenuWidgetListOutput,
};
use okshell_config::schema::menu_widgets::{ContainerConfig, MenuWidget};
use okshell_config::schema::position::Orientation;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, gtk};

#[derive(Debug)]
pub struct ContainerConfigModel {
    spacing: i32,
    minimum_width: i32,
    orientation: Orientation,
    widget_list: Controller<MenuWidgetListModel>,
}

#[derive(Debug)]
pub enum ContainerConfigInput {
    SpacingChanged(i32),
    MinWidthChanged(i32),
    OrientationChanged(Orientation),
    WidgetsChanged(Vec<MenuWidget>),
}

#[derive(Debug)]
pub enum ContainerConfigOutput {
    SpacingChanged(i32),
    MinWidthChanged(i32),
    OrientationChanged(Orientation),
    WidgetsChanged(Vec<MenuWidget>),
}

#[relm4::component(pub)]
impl Component for ContainerConfigModel {
    type CommandOutput = ();
    type Input = ContainerConfigInput;
    type Output = ContainerConfigOutput;
    type Init = ContainerConfig;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 4,
            add_css_class: "container-config",

            // Spacing
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,

                gtk::Label {
                    add_css_class: "label-small",
                    set_label: "Spacing",
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                #[name = "spacing_spin"]
                gtk::SpinButton {
                    set_range: (0.0, 100.0),
                    set_increments: (1.0, 10.0),
                    #[watch]
                    set_value: model.spacing as f64,
                    connect_value_changed[sender] => move |s| {
                        sender.input(ContainerConfigInput::SpacingChanged(s.value() as i32));
                    },
                },
            },

            // Min width
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,

                gtk::Label {
                    add_css_class: "label-small",
                    set_label: "Min width",
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                #[name = "width_spin"]
                gtk::SpinButton {
                    set_range: (0.0, 2000.0),
                    set_increments: (1.0, 10.0),
                    #[watch]
                    set_value: model.minimum_width as f64,
                    connect_value_changed[sender] => move |s| {
                        sender.input(ContainerConfigInput::MinWidthChanged(s.value() as i32));
                    },
                },
            },

            // Orientation
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,

                gtk::Label {
                    add_css_class: "label-small",
                    set_label: "Orientation",
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                #[name = "orientation_dd"]
                gtk::DropDown {
                    set_valign: gtk::Align::Center,
                    set_model: Some(&gtk::StringList::new(&["Horizontal", "Vertical"])),
                    #[watch]
                    set_selected: match model.orientation {
                        Orientation::Horizontal => 0,
                        Orientation::Vertical => 1,
                    },
                    connect_selected_notify[sender] => move |dd| {
                        let orientation = match dd.selected() {
                            0 => Orientation::Horizontal,
                            _ => Orientation::Vertical,
                        };
                        sender.input(ContainerConfigInput::OrientationChanged(orientation));
                    },
                },
            },

            model.widget_list.widget().clone() {},
        }
    }

    fn init(
        config: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widget_list = MenuWidgetListModel::builder()
            .launch(MenuWidgetListInit {
                widgets: config.widgets.clone(),
                draw_border: false,
            })
            .forward(sender.input_sender(), |output| match output {
                MenuWidgetListOutput::Changed(widgets) => {
                    ContainerConfigInput::WidgetsChanged(widgets)
                }
            });

        let model = ContainerConfigModel {
            spacing: config.spacing,
            minimum_width: config.minimum_width,
            orientation: config.orientation,
            widget_list,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            ContainerConfigInput::SpacingChanged(s) => {
                self.spacing = s;
                let _ = sender.output(ContainerConfigOutput::SpacingChanged(s));
            }
            ContainerConfigInput::MinWidthChanged(w) => {
                self.minimum_width = w;
                let _ = sender.output(ContainerConfigOutput::MinWidthChanged(w));
            }
            ContainerConfigInput::OrientationChanged(o) => {
                self.orientation = o.clone();
                let _ = sender.output(ContainerConfigOutput::OrientationChanged(o));
            }
            ContainerConfigInput::WidgetsChanged(w) => {
                let _ = sender.output(ContainerConfigOutput::WidgetsChanged(w));
            }
        }
    }
}
