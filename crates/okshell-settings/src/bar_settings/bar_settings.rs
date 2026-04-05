use reactive_graph::prelude::{Get, GetUntracked};
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use relm4::factory::{DynamicIndex, FactoryVecDeque};
use relm4::gtk::{gdk, gio, glib};
use relm4::gtk::prelude::*;
use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::bar_widgets::BarWidget;
use okshell_config::schema::config::{BarsStoreFields, ConfigStoreFields, FrameStoreFields, HorizontalBarStoreFields, VerticalBarStoreFields};
use crate::bar_settings::bar_widget_section::{BarSection, WidgetSectionInit, WidgetSectionModel, WidgetSectionOutput};
use crate::bar_settings::monitor_chip::{MonitorChipModel, MonitorChipOutput};

#[derive(Debug)]
pub(crate) struct BarSettingsModel {
    enable_frame: bool,
    chips: FactoryVecDeque<MonitorChipModel>,
    available_monitors: Vec<String>,
    selected_monitors: Vec<String>,
    top_bar_start_controller: Controller<WidgetSectionModel>,
    top_bar_center_controller: Controller<WidgetSectionModel>,
    top_bar_end_controller: Controller<WidgetSectionModel>,
    left_bar_start_controller: Controller<WidgetSectionModel>,
    left_bar_center_controller: Controller<WidgetSectionModel>,
    left_bar_end_controller: Controller<WidgetSectionModel>,
    right_bar_start_controller: Controller<WidgetSectionModel>,
    right_bar_center_controller: Controller<WidgetSectionModel>,
    right_bar_end_controller: Controller<WidgetSectionModel>,
    bottom_bar_start_controller: Controller<WidgetSectionModel>,
    bottom_bar_center_controller: Controller<WidgetSectionModel>,
    bottom_bar_end_controller: Controller<WidgetSectionModel>,
    top_min_height: i32,
    bottom_min_height: i32,
    left_min_width: i32,
    right_min_width: i32,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum BarSettingsInput {
    EnableFrameToggled(bool),
    EnableFrameChanged(bool),
    AddMonitor(String),
    RemoveMonitor(DynamicIndex),
    AvailableMonitorsChanged(Vec<String>),
    SelectedMonitorsChanged(Vec<String>),
    TopStartChanged(Vec<BarWidget>),
    TopCenterChanged(Vec<BarWidget>),
    TopEndChanged(Vec<BarWidget>),
    BottomStartChanged(Vec<BarWidget>),
    BottomCenterChanged(Vec<BarWidget>),
    BottomEndChanged(Vec<BarWidget>),
    LeftStartChanged(Vec<BarWidget>),
    LeftCenterChanged(Vec<BarWidget>),
    LeftEndChanged(Vec<BarWidget>),
    RightStartChanged(Vec<BarWidget>),
    RightCenterChanged(Vec<BarWidget>),
    RightEndChanged(Vec<BarWidget>),
    TopMinHeightChanged(i32),
    BottomMinHeightChanged(i32),
    RightMinWidthChanged(i32),
    LeftMinWidthChanged(i32),
}

#[derive(Debug)]
pub(crate) enum BarSettingsOutput {}

pub(crate) struct BarSettingsInit {}

#[derive(Debug)]
pub(crate) enum BarSettingsCommandOutput {}

#[relm4::component(pub)]
impl Component for BarSettingsModel {
    type CommandOutput = BarSettingsCommandOutput;
    type Input = BarSettingsInput;
    type Output = BarSettingsOutput;
    type Init = BarSettingsInit;

    view! {
        #[root]
        gtk::ScrolledWindow {
            set_vscrollbar_policy: gtk::PolicyType::Automatic,
            set_hscrollbar_policy: gtk::PolicyType::Never,
            set_propagate_natural_height: false,
            set_propagate_natural_width: false,
            set_hexpand: true,
            set_vexpand: true,

            gtk::Box {
                add_css_class: "settings-page",
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_spacing: 16,

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Frame",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Label {
                        add_css_class: "label-small",
                        set_halign: gtk::Align::Start,
                        set_label: "Enable frame drawing.",
                        set_hexpand: true,
                    },

                    gtk::Switch {
                        #[watch]
                        #[block_signal(enable_frame_handler)]
                        set_active: model.enable_frame,
                        connect_state_set[sender] => move |_, enabled| {
                            sender.input(BarSettingsInput::EnableFrameToggled(enabled));
                            glib::Propagation::Proceed
                        } @enable_frame_handler,
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Monitors",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "Monitors to show the frame on. If empty, a frame will show on all monitors.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },

                        // Empty state
                        gtk::Label {
                            #[watch]
                            set_visible: model.selected_monitors.is_empty(),
                            set_label: "All monitors",
                            set_halign: gtk::Align::Start,
                            set_css_classes: &["monitor-chip-empty", "label-small"],
                        },

                        // Chips
                        #[local_ref]
                        chip_box -> gtk::FlowBox {
                            set_selection_mode: gtk::SelectionMode::None,
                            set_row_spacing: 4,
                            set_column_spacing: 4,
                            set_homogeneous: false,
                            #[watch]
                            set_visible: !model.selected_monitors.is_empty(),
                        },
                    },

                    // Add button
                    #[name = "add_monitor_button"]
                    gtk::MenuButton {
                        set_label: "Add monitor",
                        add_css_class: "ok-button-primary",
                        set_vexpand: false,
                        set_hexpand: false,
                        set_halign: gtk::Align::Start,
                        set_valign: gtk::Align::Start,
                        #[watch]
                        set_sensitive: model.has_unselected_monitors(),
                    },
                },

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Top Bar",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Label {
                        add_css_class: "label-small",
                        set_halign: gtk::Align::Start,
                        set_label: "Minimum Height",
                        set_hexpand: true,
                    },

                    gtk::SpinButton {
                        set_range: (0.0, 500.0),
                        set_increments: (1.0, 10.0),
                        #[watch]
                        set_value: model.top_min_height as f64,
                        connect_value_changed[sender] => move |s| {
                            sender.input(BarSettingsInput::TopMinHeightChanged(s.value() as i32));
                        },
                    },
                },

                model.top_bar_start_controller.widget().clone() {},
                model.top_bar_center_controller.widget().clone() {},
                model.top_bar_end_controller.widget().clone() {},

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Left Bar",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Label {
                        add_css_class: "label-small",
                        set_halign: gtk::Align::Start,
                        set_label: "Minimum Width",
                        set_hexpand: true,
                    },

                    gtk::SpinButton {
                        set_range: (0.0, 500.0),
                        set_increments: (1.0, 10.0),
                        #[watch]
                        set_value: model.left_min_width as f64,
                        connect_value_changed[sender] => move |s| {
                            sender.input(BarSettingsInput::LeftMinWidthChanged(s.value() as i32));
                        },
                    },
                },

                model.left_bar_start_controller.widget().clone() {},
                model.left_bar_center_controller.widget().clone() {},
                model.left_bar_end_controller.widget().clone() {},

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Right Bar",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Label {
                        add_css_class: "label-small",
                        set_halign: gtk::Align::Start,
                        set_label: "Minimum Width",
                        set_hexpand: true,
                    },

                    gtk::SpinButton {
                        set_range: (0.0, 500.0),
                        set_increments: (1.0, 10.0),
                        #[watch]
                        set_value: model.right_min_width as f64,
                        connect_value_changed[sender] => move |s| {
                            sender.input(BarSettingsInput::RightMinWidthChanged(s.value() as i32));
                        },
                    },
                },

                model.right_bar_start_controller.widget().clone() {},
                model.right_bar_center_controller.widget().clone() {},
                model.right_bar_end_controller.widget().clone() {},

                gtk::Separator {},

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Bottom Bar",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Label {
                        add_css_class: "label-small",
                        set_halign: gtk::Align::Start,
                        set_label: "Minimum Height",
                        set_hexpand: true,
                    },

                    gtk::SpinButton {
                        set_range: (0.0, 500.0),
                        set_increments: (1.0, 10.0),
                        #[watch]
                        set_value: model.bottom_min_height as f64,
                        connect_value_changed[sender] => move |s| {
                            sender.input(BarSettingsInput::BottomMinHeightChanged(s.value() as i32));
                        },
                    },
                },

                model.bottom_bar_start_controller.widget().clone() {},
                model.bottom_bar_center_controller.widget().clone() {},
                model.bottom_bar_end_controller.widget().clone() {},
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let chips = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::default())
            .forward(sender.input_sender(), |output| match output {
                MonitorChipOutput::Remove(index) => BarSettingsInput::RemoveMonitor(index),
            });

        let mut effects = EffectScope::new();

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let enabled = config.bars().frame().enable_frame().get();
            sender_clone.input(BarSettingsInput::EnableFrameChanged(enabled));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let monitors = config.bars().frame().monitor_filter().get();
            sender_clone.input(BarSettingsInput::SelectedMonitorsChanged(monitors));
        });

        let sender_clone = sender.clone();
        if let Some(display) = gdk::Display::default() {
            let monitors = display.monitors();
            let names: Vec<String> = (0..monitors.n_items())
                .filter_map(|i| monitors.item(i))
                .filter_map(|obj| obj.downcast::<gdk::Monitor>().ok())
                .filter_map(|m| m.connector().map(|c| c.to_string()))
                .collect();
            sender_clone.input(BarSettingsInput::AvailableMonitorsChanged(names));

            // Also listen for monitor changes
            let sender_clone2 = sender.clone();
            display.connect_notify(Some("monitors"), move |display, _| {
                let monitors = display.monitors();
                let names: Vec<String> = (0..monitors.n_items())
                    .filter_map(|i| monitors.item(i))
                    .filter_map(|obj| obj.downcast::<gdk::Monitor>().ok())
                    .filter_map(|m| m.connector().map(|c| c.to_string()))
                    .collect();
                sender_clone2.input(BarSettingsInput::AvailableMonitorsChanged(names));
            });
        }

        let top_bar_start_controller = WidgetSectionModel::builder()
            .launch(WidgetSectionInit {
                bar_section: BarSection::Start,
                widgets: config_manager().config().bars().top_bar().left_widgets().get_untracked(),
            })
            .forward(sender.input_sender(), |msg| {
                match msg { WidgetSectionOutput::Changed(widgets) => {
                    BarSettingsInput::TopStartChanged(widgets)
                } }
            });

        let top_bar_center_controller = WidgetSectionModel::builder()
            .launch(WidgetSectionInit {
                bar_section: BarSection::Center,
                widgets: config_manager().config().bars().top_bar().center_widgets().get_untracked(),
            })
            .forward(sender.input_sender(), |msg| {
                match msg { WidgetSectionOutput::Changed(widgets) => {
                    BarSettingsInput::TopCenterChanged(widgets)
                } }
            });

        let top_bar_end_controller = WidgetSectionModel::builder()
            .launch(WidgetSectionInit {
                bar_section: BarSection::End,
                widgets: config_manager().config().bars().top_bar().right_widgets().get_untracked(),
            })
            .forward(sender.input_sender(), |msg| {
                match msg { WidgetSectionOutput::Changed(widgets) => {
                    BarSettingsInput::TopEndChanged(widgets)
                } }
            });

        let left_bar_start_controller = WidgetSectionModel::builder()
            .launch(WidgetSectionInit {
                bar_section: BarSection::Start,
                widgets: config_manager().config().bars().left_bar().top_widgets().get_untracked(),
            })
            .forward(sender.input_sender(), |msg| {
                match msg { WidgetSectionOutput::Changed(widgets) => {
                    BarSettingsInput::LeftStartChanged(widgets)
                } }
            });

        let left_bar_center_controller = WidgetSectionModel::builder()
            .launch(WidgetSectionInit {
                bar_section: BarSection::Center,
                widgets: config_manager().config().bars().left_bar().center_widgets().get_untracked(),
            })
            .forward(sender.input_sender(), |msg| {
                match msg { WidgetSectionOutput::Changed(widgets) => {
                    BarSettingsInput::LeftCenterChanged(widgets)
                } }
            });

        let left_bar_end_controller = WidgetSectionModel::builder()
            .launch(WidgetSectionInit {
                bar_section: BarSection::End,
                widgets: config_manager().config().bars().left_bar().bottom_widgets().get_untracked(),
            })
            .forward(sender.input_sender(), |msg| {
                match msg { WidgetSectionOutput::Changed(widgets) => {
                    BarSettingsInput::LeftEndChanged(widgets)
                } }
            });

        let right_bar_start_controller = WidgetSectionModel::builder()
            .launch(WidgetSectionInit {
                bar_section: BarSection::Start,
                widgets: config_manager().config().bars().right_bar().top_widgets().get_untracked(),
            })
            .forward(sender.input_sender(), |msg| {
                match msg { WidgetSectionOutput::Changed(widgets) => {
                    BarSettingsInput::RightStartChanged(widgets)
                } }
            });

        let right_bar_center_controller = WidgetSectionModel::builder()
            .launch(WidgetSectionInit {
                bar_section: BarSection::Center,
                widgets: config_manager().config().bars().right_bar().center_widgets().get_untracked(),
            })
            .forward(sender.input_sender(), |msg| {
                match msg { WidgetSectionOutput::Changed(widgets) => {
                    BarSettingsInput::RightCenterChanged(widgets)
                } }
            });

        let right_bar_end_controller = WidgetSectionModel::builder()
            .launch(WidgetSectionInit {
                bar_section: BarSection::End,
                widgets: config_manager().config().bars().right_bar().bottom_widgets().get_untracked(),
            })
            .forward(sender.input_sender(), |msg| {
                match msg { WidgetSectionOutput::Changed(widgets) => {
                    BarSettingsInput::RightEndChanged(widgets)
                } }
            });

        let bottom_bar_start_controller = WidgetSectionModel::builder()
            .launch(WidgetSectionInit {
                bar_section: BarSection::Start,
                widgets: config_manager().config().bars().bottom_bar().left_widgets().get_untracked(),
            })
            .forward(sender.input_sender(), |msg| {
                match msg { WidgetSectionOutput::Changed(widgets) => {
                    BarSettingsInput::BottomStartChanged(widgets)
                } }
            });

        let bottom_bar_center_controller = WidgetSectionModel::builder()
            .launch(WidgetSectionInit {
                bar_section: BarSection::Center,
                widgets: config_manager().config().bars().bottom_bar().center_widgets().get_untracked(),
            })
            .forward(sender.input_sender(), |msg| {
                match msg { WidgetSectionOutput::Changed(widgets) => {
                    BarSettingsInput::BottomCenterChanged(widgets)
                } }
            });

        let bottom_bar_end_controller = WidgetSectionModel::builder()
            .launch(WidgetSectionInit {
                bar_section: BarSection::End,
                widgets: config_manager().config().bars().bottom_bar().right_widgets().get_untracked(),
            })
            .forward(sender.input_sender(), |msg| {
                match msg { WidgetSectionOutput::Changed(widgets) => {
                    BarSettingsInput::BottomEndChanged(widgets)
                } }
            });

        let model = BarSettingsModel {
            enable_frame: false,
            chips,
            available_monitors: Vec::new(),
            selected_monitors: Vec::new(),
            top_bar_start_controller,
            top_bar_center_controller,
            top_bar_end_controller,
            left_bar_start_controller,
            left_bar_center_controller,
            left_bar_end_controller,
            right_bar_start_controller,
            right_bar_center_controller,
            right_bar_end_controller,
            bottom_bar_start_controller,
            bottom_bar_center_controller,
            bottom_bar_end_controller,
            top_min_height: config_manager().config().bars().top_bar().minimum_height().get_untracked(),
            bottom_min_height: config_manager().config().bars().bottom_bar().minimum_height().get_untracked(),
            left_min_width: config_manager().config().bars().left_bar().minimum_width().get_untracked(),
            right_min_width: config_manager().config().bars().right_bar().minimum_width().get_untracked(),
            _effects: effects,
        };

        let chip_box = model.chips.widget();

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
            BarSettingsInput::EnableFrameToggled(enabled) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.frame.enable_frame = enabled;
                });
            }
            BarSettingsInput::EnableFrameChanged(enable) => {
                self.enable_frame = enable;
            }
            BarSettingsInput::AddMonitor(name) => {
                if !self.selected_monitors.contains(&name) {
                    self.selected_monitors.push(name.clone());
                    self.chips.guard().push_back(name);
                    config_manager().update_config(|config| {
                        config.bars.frame.monitor_filter = self.selected_monitors.clone();
                    });
                }
                self.rebuild_menu(widgets, &sender);
            }
            BarSettingsInput::RemoveMonitor(index) => {
                let idx = index.current_index();
                if idx < self.selected_monitors.len() {
                    self.selected_monitors.remove(idx);
                    self.chips.guard().remove(idx);
                    config_manager().update_config(|config| {
                        config.bars.frame.monitor_filter = self.selected_monitors.clone();
                    });
                }
                self.rebuild_menu(widgets, &sender);
            }
            BarSettingsInput::AvailableMonitorsChanged(monitors) => {
                self.available_monitors = monitors;
                self.rebuild_menu(widgets, &sender);
            }
            BarSettingsInput::SelectedMonitorsChanged(monitors) => {
                self.selected_monitors = monitors.clone();
                let mut guard = self.chips.guard();
                guard.clear();
                for name in monitors {
                    guard.push_back(name);
                }
                drop(guard);
                self.rebuild_menu(widgets, &sender);
            }
            BarSettingsInput::TopStartChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.top_bar.left_widgets = widgets;
                });
            }
            BarSettingsInput::TopCenterChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.top_bar.center_widgets = widgets;
                });
            }
            BarSettingsInput::TopEndChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.top_bar.right_widgets = widgets;
                });
            }
            BarSettingsInput::LeftStartChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.left_bar.top_widgets = widgets;
                });
            }
            BarSettingsInput::LeftCenterChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.left_bar.center_widgets = widgets;
                });
            }
            BarSettingsInput::LeftEndChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.left_bar.bottom_widgets = widgets;
                });
            }
            BarSettingsInput::RightStartChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.right_bar.top_widgets = widgets;
                });
            }
            BarSettingsInput::RightCenterChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.right_bar.center_widgets = widgets;
                });
            }
            BarSettingsInput::RightEndChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.right_bar.bottom_widgets = widgets;
                });
            }
            BarSettingsInput::BottomStartChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.bottom_bar.left_widgets = widgets;
                });
            }
            BarSettingsInput::BottomCenterChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.bottom_bar.center_widgets = widgets;
                });
            }
            BarSettingsInput::BottomEndChanged(widgets) => {
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.bottom_bar.right_widgets = widgets;
                });
            }
            BarSettingsInput::TopMinHeightChanged(min) => {
                self.top_min_height = min;
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.top_bar.minimum_height = min;
                });
            }
            BarSettingsInput::BottomMinHeightChanged(min) => {
                self.bottom_min_height = min;
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.bottom_bar.minimum_height = min;
                });
            }
            BarSettingsInput::LeftMinWidthChanged(min) => {
                self.left_min_width = min;
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.left_bar.minimum_width = min;
                });
            }
            BarSettingsInput::RightMinWidthChanged(min) => {
                self.right_min_width = min;
                let config_manager = config_manager();
                config_manager.update_config(|config| {
                    config.bars.right_bar.minimum_width = min;
                });
            }
        }

        self.update_view(widgets, sender);
    }
}

impl BarSettingsModel {
    fn has_unselected_monitors(&self) -> bool {
        self.available_monitors.iter().any(|m| !self.selected_monitors.contains(m))
    }

    fn rebuild_menu(
        &self,
        widgets: &<Self as Component>::Widgets,
        sender: &ComponentSender<Self>,
    ) {
        let menu = gio::Menu::new();
        let action_group = gio::SimpleActionGroup::new();

        for name in &self.available_monitors {
            if self.selected_monitors.contains(name) {
                continue;
            }

            let action_name = format!("add-{}", name.replace(' ', "-"));
            let action = gio::SimpleAction::new(&action_name, None);

            let sender = sender.input_sender().clone();
            let monitor_name = name.clone();
            action.connect_activate(move |_, _| {
                sender.emit(BarSettingsInput::AddMonitor(monitor_name.clone()));
            });

            action_group.add_action(&action);
            menu.append(Some(name.as_str()), Some(&format!("monitor.{action_name}")));
        }

        widgets.add_monitor_button.insert_action_group("monitor", Some(&action_group));
        widgets.add_monitor_button.set_menu_model(Some(&menu));
    }
}