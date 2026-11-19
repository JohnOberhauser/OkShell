use okshell_services::sys_info_service;
use okshell_utils::sys_info::spawn_memory_watcher;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};

use crate::common_widgets::usage_ring::UsageRing;

#[derive(Debug, Clone)]
pub(crate) struct MemoryUsageModel {
    ring: UsageRing,
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
            set_css_classes: &["memory-usage-bar-widget", "ok-button-surface", "ok-bar-widget"],
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
        spawn_memory_watcher(&sender, || MemoryUsageCommandOutput::MemoryStateChanged);

        let model = MemoryUsageModel {
            ring: UsageRing::new("memory-symbolic"),
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
            MemoryUsageCommandOutput::MemoryStateChanged => {
                let usage = sys_info_service().memory.get().usage_percent as f64;
                self.ring.set_value(usage);
            }
        }
    }
}
