use std::rc::Rc;

use okshell_services::sys_info_service;
use okshell_utils::sys_info::spawn_memory_watcher;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};

use super::level_anim::{LevelAnim, percent_of}; // adjust path

const BYTES_PER_GB: f64 = 1024.0 * 1024.0 * 1024.0;

#[derive(Debug, Clone)]
pub(crate) struct MemoryUsageModel {
    used_bar: Rc<LevelAnim>,
    total_label: String,
    used_label: String,
    free_label: String,
}

#[derive(Debug)]
pub(crate) enum MemoryUsageInput {}

#[derive(Debug)]
pub(crate) enum MemoryUsageOutput {}

pub(crate) struct MemoryUsageInit {}

#[derive(Debug)]
pub(crate) enum MemoryUsageCommandOutput {
    MemoryStateChanged,
}

#[relm4::component(pub)]
impl Component for MemoryUsageModel {
    type CommandOutput = MemoryUsageCommandOutput;
    type Input = MemoryUsageInput;
    type Output = MemoryUsageOutput;
    type Init = MemoryUsageInit;

    view! {
        #[root]
        gtk::Box {
            set_css_classes: &["memory-usage-menu-widget"],
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,
            set_hexpand: true,

            gtk::Label {
                add_css_class: "label-large-bold",
                set_label: "Memory",
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,

                gtk::Label {
                    add_css_class: "label-medium-bold",
                    set_label: "Total:",
                    set_margin_end: 12,
                },

                gtk::Label {
                    add_css_class: "label-medium",
                    #[watch]
                    set_label: model.total_label.as_str(),
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_spacing: 4,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,

                    gtk::Label {
                        add_css_class: "label-medium-bold",
                        set_label: "Used:",
                        set_margin_end: 12,
                    },

                    gtk::Label {
                        add_css_class: "label-medium",
                        #[watch]
                        set_label: model.used_label.as_str(),
                    },

                    // Takes the leftover space and pushes "Free" to the far right.
                    gtk::Box {
                        set_hexpand: true,
                    },

                    gtk::Label {
                        add_css_class: "label-medium-bold",
                        set_label: "Free:",
                        set_margin_end: 12,
                    },

                    gtk::Label {
                        add_css_class: "label-medium",
                        #[watch]
                        set_label: model.free_label.as_str(),
                    },
                },

                #[local_ref]
                used_bar -> gtk::LevelBar {
                    add_css_class: "memory-used-bar",
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_memory_watcher(&sender, || MemoryUsageCommandOutput::MemoryStateChanged);

        let mut model = MemoryUsageModel {
            used_bar: LevelAnim::new(),
            total_label: String::new(),
            used_label: String::new(),
            free_label: String::new(),
        };

        // Fill labels and bar right away instead of waiting for the first update.
        model.refresh();

        let used_bar = model.used_bar.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            MemoryUsageCommandOutput::MemoryStateChanged => {
                self.refresh();
            }
        }
        self.update_view(widgets, sender);
    }
}

impl MemoryUsageModel {
    fn refresh(&mut self) {
        let memory = sys_info_service().memory.get();

        let total = memory.total_bytes as f64;
        let used = memory.used_bytes as f64;
        let free = memory.available_bytes as f64;

        self.total_label = format!("{:.0} GB", total / BYTES_PER_GB);
        self.used_label = format!("{:.1} GB", used / BYTES_PER_GB);
        self.free_label = format!("{:.1} GB", free / BYTES_PER_GB);

        self.used_bar.animate_to(percent_of(used, total));
    }
}
