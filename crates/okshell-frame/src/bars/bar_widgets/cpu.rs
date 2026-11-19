use okshell_services::sys_info_service;
use okshell_utils::sys_info::spawn_cpu_watcher;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};

use crate::common_widgets::usage_ring::UsageRing;

#[derive(Debug, Clone)]
pub(crate) struct CpuUsageModel {
    ring: UsageRing,
}

#[derive(Debug)]
pub(crate) enum CpuUsageInput {}

#[derive(Debug)]
pub(crate) enum CpuUsageOutput {}

pub(crate) struct CpuUsageInit {}

#[derive(Debug)]
pub(crate) enum CpuUsageCommandOutput {
    CpuStateChanged,
}

#[relm4::component(pub)]
impl Component for CpuUsageModel {
    type CommandOutput = CpuUsageCommandOutput;
    type Input = CpuUsageInput;
    type Output = CpuUsageOutput;
    type Init = CpuUsageInit;

    view! {
        #[root]
        gtk::Box {
            set_css_classes: &["cpu-usage-bar-widget", "ok-button-surface", "ok-bar-widget"],
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
        spawn_cpu_watcher(&sender, || CpuUsageCommandOutput::CpuStateChanged);

        let model = CpuUsageModel {
            ring: UsageRing::new("cpu-symbolic"),
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
            CpuUsageCommandOutput::CpuStateChanged => {
                let usage = sys_info_service().cpu.get().usage_percent as f64;
                self.ring.set_value(usage);
            }
        }
    }
}
