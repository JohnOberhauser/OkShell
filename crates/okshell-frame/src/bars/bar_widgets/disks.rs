use std::path::Path;

use okshell_services::sys_info_service;
use okshell_utils::sys_info::spawn_disk_watcher;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};

use crate::common_widgets::usage_ring::UsageRing;

#[derive(Debug, Clone)]
pub(crate) struct DisksUsageModel {
    ring: UsageRing,
}

#[derive(Debug)]
pub(crate) enum DisksUsageInput {}

#[derive(Debug)]
pub(crate) enum DisksUsageOutput {}

pub(crate) struct DisksUsageInit {}

#[derive(Debug)]
pub(crate) enum DisksUsageCommandOutput {
    DisksStateChanged,
}

#[relm4::component(pub)]
impl Component for DisksUsageModel {
    type CommandOutput = DisksUsageCommandOutput;
    type Input = DisksUsageInput;
    type Output = DisksUsageOutput;
    type Init = DisksUsageInit;

    view! {
        #[root]
        gtk::Box {
            set_css_classes: &["disk-usage-bar-widget", "ok-button-surface", "ok-bar-widget"],
            set_hexpand: false,
            set_vexpand: false,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,

            #[local_ref]
            ring_widget -> gtk::Overlay {},
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_disk_watcher(&sender, || DisksUsageCommandOutput::DisksStateChanged);

        let model = DisksUsageModel {
            ring: UsageRing::new("disk-symbolic"),
        };

        let ring_widget = model.ring.widget();
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
            DisksUsageCommandOutput::DisksStateChanged => {
                let disks = sys_info_service().disks.get();
                if let Some(disk) = disks.iter().find(|d| d.mount_point == Path::new("/")) {
                    let usage = disk.usage_percent as f64;
                    self.ring.set_value(usage);
                }
            }
        }
    }
}
