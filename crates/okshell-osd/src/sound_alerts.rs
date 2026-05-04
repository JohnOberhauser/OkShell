use gtk4::glib;
use gtk4::prelude::{GtkWindowExt, WidgetExt};
use gtk4_layer_shell::{Layer, LayerShell};
use okshell_services::{battery_service, line_power_service};
use okshell_sounds::{play_battery_low, play_power_plug_sound, play_power_unplug_sound};
use okshell_utils::battery::{spawn_battery_online_watcher, spawn_battery_watcher};
use relm4::{Component, ComponentParts, ComponentSender, gtk};
use wayle_battery::types::DeviceState;

#[derive(Debug)]
pub struct SoundAlertsModel {
    battery_sound_tick_source: Option<glib::SourceId>,
    line_power_first_skipped: bool,
}

#[derive(Debug)]
pub enum SoundAlertsInput {
    Tick,
}

#[derive(Debug)]
pub enum SoundAlertsOutput {}

#[derive(Debug)]
pub enum SoundAlertsCommandOutput {
    BatteryChanged,
    BatteryTypeChanged,
}

#[relm4::component(pub)]
impl Component for SoundAlertsModel {
    type CommandOutput = SoundAlertsCommandOutput;
    type Input = SoundAlertsInput;
    type Output = SoundAlertsOutput;
    type Init = ();

    view! {
        #[root]
        gtk::Window {
            add_css_class: "sound-alerts",
            set_decorated: false,
            set_visible: false,
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.init_layer_shell();
        root.set_layer(Layer::Background);
        root.set_exclusive_zone(-1);

        spawn_battery_watcher(&sender, || SoundAlertsCommandOutput::BatteryChanged);

        spawn_battery_online_watcher(&sender, || SoundAlertsCommandOutput::BatteryTypeChanged);

        let model = SoundAlertsModel {
            battery_sound_tick_source: None,
            line_power_first_skipped: false,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            SoundAlertsInput::Tick => {
                play_battery_low();
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            SoundAlertsCommandOutput::BatteryChanged => {
                let battery_service = battery_service();
                let device = battery_service.device.clone();
                let discharging_low = device.is_present.get()
                    && device.state.get() == DeviceState::Discharging
                    && device.percentage.get() < 4.0;

                if discharging_low && self.battery_sound_tick_source.is_none() {
                    // Fire immediately, then every 60s
                    sender.input(SoundAlertsInput::Tick);

                    let tick_sender = sender.input_sender().clone();
                    let id = glib::timeout_add_seconds_local(60, move || {
                        let _ = tick_sender.send(SoundAlertsInput::Tick);
                        glib::ControlFlow::Continue
                    });
                    self.battery_sound_tick_source = Some(id);
                } else if !discharging_low && let Some(id) = self.battery_sound_tick_source.take() {
                    id.remove();
                }
            }
            SoundAlertsCommandOutput::BatteryTypeChanged => {
                if let Some(service) = line_power_service() {
                    if !self.line_power_first_skipped {
                        self.line_power_first_skipped = true;
                        return;
                    }
                    let online = service.device.online.get();
                    if online {
                        play_power_plug_sound();
                    } else {
                        play_power_unplug_sound();
                    }
                }
            }
        }
    }
}
