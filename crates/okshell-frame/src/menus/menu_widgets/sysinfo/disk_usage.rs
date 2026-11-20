use okshell_services::sys_info_service;
use okshell_utils::sys_info::spawn_disk_watcher;
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryVecDeque};
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};
use std::path::PathBuf;
use std::rc::Rc;
use wayle_sysinfo::types::DiskData;

use super::level_anim::{LevelAnim, percent_of}; // adjust path

const BYTES_PER_GB: f64 = 1024.0 * 1024.0 * 1024.0;

// ---------------------------------------------------------------------------
// Parent: "Disks" title + list of disks
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub(crate) struct DiskUsageModel {
    disks: FactoryVecDeque<DiskItemModel>,
}

#[derive(Debug)]
pub(crate) enum DiskUsageInput {}

#[derive(Debug)]
pub(crate) enum DiskUsageOutput {}

pub(crate) struct DiskUsageInit {}

#[derive(Debug)]
pub(crate) enum DiskUsageCommandOutput {
    DisksStateChanged,
}

#[relm4::component(pub)]
impl Component for DiskUsageModel {
    type CommandOutput = DiskUsageCommandOutput;
    type Input = DiskUsageInput;
    type Output = DiskUsageOutput;
    type Init = DiskUsageInit;

    view! {
        #[root]
        gtk::Box {
            set_css_classes: &["disk-usage-menu-widget"],
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,
            set_hexpand: true,

            gtk::CenterBox {
                set_hexpand: true,

                #[wrap(Some)]
                set_start_widget = &gtk::Image {
                    set_hexpand: true,
                    set_halign: gtk::Align::End,
                    set_valign: gtk::Align::Center,
                    set_margin_end: 8,
                    set_icon_name: Some("disk-symbolic"),
                },

                #[wrap(Some)]
                set_center_widget = &gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Disks",
                },
            },

            #[local_ref]
            disks_box -> gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 16,
                set_hexpand: true,
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_disk_watcher(&sender, || DiskUsageCommandOutput::DisksStateChanged);

        let disks = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        let mut model = DiskUsageModel { disks };

        // Fill the list right away instead of waiting for the first update.
        model.refresh();

        let disks_box = model.disks.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            DiskUsageCommandOutput::DisksStateChanged => self.refresh(),
        }
    }
}

impl DiskUsageModel {
    fn refresh(&mut self) {
        let disks: Vec<DiskData> = sys_info_service()
            .disks
            .get()
            .iter()
            .filter(|disk| !disk.mount_point.starts_with("/var"))
            .filter(|disk| !disk.mount_point.starts_with("/home"))
            .filter(|disk| !disk.mount_point.starts_with("/boot"))
            .cloned()
            .collect();

        // Same mounts in the same order: update rows in place so the bars animate.
        let same_mounts = self.disks.len() == disks.len()
            && self
                .disks
                .iter()
                .zip(disks.iter())
                .all(|(item, disk)| item.mount_point == disk.mount_point);

        if same_mounts {
            for (index, disk) in disks.into_iter().enumerate() {
                self.disks.send(index, DiskItemInput::Update(disk));
            }
        } else {
            // A disk was mounted or unmounted: rebuild the list.
            let mut guard = self.disks.guard();
            guard.clear();
            for disk in disks {
                guard.push_back(disk);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Item: one disk
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub(crate) struct DiskItemModel {
    mount_point: PathBuf,
    used_bar: Rc<LevelAnim>,
    mount_label: String,
    total_label: String,
    used_label: String,
    free_label: String,
}

#[derive(Debug)]
pub(crate) enum DiskItemInput {
    Update(DiskData),
}

#[relm4::factory(pub(crate))]
impl FactoryComponent for DiskItemModel {
    type Init = DiskData;
    type Input = DiskItemInput;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "disk-usage-item",
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,
            set_hexpand: true,

            gtk::Label {
                add_css_class: "label-medium-bold",
                #[watch]
                set_label: self.mount_label.as_str(),
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                gtk::Label {
                    add_css_class: "label-medium-bold",
                    set_label: "Total:",
                    set_margin_end: 12,
                },

                gtk::Label {
                    add_css_class: "label-medium",
                    #[watch]
                    set_label: self.total_label.as_str(),
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
                        set_label: self.used_label.as_str(),
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
                        set_label: self.free_label.as_str(),
                    },
                },

                #[local_ref]
                used_bar -> gtk::LevelBar {
                    add_css_class: "disk-used-bar",
                },
            },
        }
    }

    fn init_model(disk: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let mut model = Self {
            mount_point: disk.mount_point.clone(),
            used_bar: LevelAnim::new(),
            mount_label: String::new(),
            total_label: String::new(),
            used_label: String::new(),
            free_label: String::new(),
        };
        model.apply(&disk);
        model
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &gtk::Widget,
        _sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let used_bar = self.used_bar.widget();
        let widgets = view_output!();
        widgets
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            DiskItemInput::Update(disk) => self.apply(&disk),
        }
    }
}

impl DiskItemModel {
    fn apply(&mut self, disk: &DiskData) {
        let total = disk.total_bytes as f64;
        let used = disk.used_bytes as f64;
        let free = disk.available_bytes as f64;

        self.mount_point = disk.mount_point.clone();
        self.mount_label = disk.mount_point.display().to_string();
        self.total_label = format!("{:.0} GB", total / BYTES_PER_GB);
        self.used_label = format!("{:.1} GB", used / BYTES_PER_GB);
        self.free_label = format!("{:.1} GB", free / BYTES_PER_GB);

        self.used_bar.animate_to(percent_of(used, total));
    }
}
