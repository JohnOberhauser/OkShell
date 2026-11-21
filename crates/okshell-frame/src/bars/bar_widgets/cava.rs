use futures::StreamExt;
use okshell_config::schema::bar_widgets::CavaConfig;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};
use tokio::sync::watch;
use tracing::error;
use wayle_cava::{CavaService, InputMethod};

const BAR_SPACING: i32 = 1;
/// Cava values are normalized to 0.0..=1.0.
const MAX_VALUE: f64 = 1.0;

/// Settings not exposed in `CavaConfig`.
const FRAMERATE: u32 = 30;
const INPUT: InputMethod = InputMethod::PipeWire;
const SOURCE: &str = "auto";

#[derive(Debug)]
pub(crate) struct CavaModel {
    /// Latest desired config. The service task watches this and applies
    /// changes one at a time, so rapid updates coalesce and restarts never overlap.
    config: watch::Sender<CavaConfig>,
    orientation: gtk::Orientation,
    bars: Vec<CavaBar>,
}

#[derive(Debug, Clone)]
struct CavaBar {
    container: gtk::Box,
    start: gtk::LevelBar,
    end: gtk::LevelBar,
}

#[derive(Debug)]
pub(crate) enum CavaInput {
    SetConfig(CavaConfig),
}

#[derive(Debug)]
pub(crate) enum CavaOutput {}

pub(crate) struct CavaInit {
    /// Orientation of the stack of bars. Horizontal lays the bars side by side
    /// and they grow vertically; vertical stacks them and they grow horizontally.
    pub(crate) orientation: gtk::Orientation,
    pub(crate) config: CavaConfig,
}

#[derive(Debug)]
pub(crate) enum CavaCommandOutput {
    Values(Vec<f64>),
}

#[relm4::component(pub)]
impl Component for CavaModel {
    type CommandOutput = CavaCommandOutput;
    type Input = CavaInput;
    type Output = CavaOutput;
    type Init = CavaInit;

    view! {
        #[root]
        #[name = "root"]
        gtk::Box {
            set_css_classes: &["cava-bar-widget", "ok-bar-widget"],
            set_hexpand: false,
            set_vexpand: false,

            // The bar's `.ok-bar-widget > *` padding lands on this box,
            // not on each individual bar.
            #[name = "bars"]
            gtk::Box {
                add_css_class: "cava-bars",
                set_orientation: model.orientation,
                set_spacing: BAR_SPACING,
                set_hexpand: true,
                set_vexpand: true,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (config_tx, mut config_rx) = watch::channel(params.config);

        let model = CavaModel {
            config: config_tx,
            orientation: params.orientation,
            bars: Vec::new(),
        };

        let widgets = view_output!();

        // Fill the available space along the direction the bars move, and stay
        // centered along the direction they're stacked.
        match model.orientation {
            gtk::Orientation::Vertical => widgets.bars.set_halign(gtk::Align::Fill),
            _ => widgets.bars.set_valign(gtk::Align::Fill),
        }

        // The service lives entirely inside this task. When the component is
        // destroyed the task is dropped, which drops the service and stops cava.
        // GTK widgets can't be touched from here, so values go back to the main
        // thread as command output.
        sender.command(|out, shutdown| {
            shutdown
                .register(async move {
                    let mut applied = config_rx.borrow_and_update().clone();
                    let service = match build_service(&applied).await {
                        Ok(service) => service,
                        Err(err) => {
                            error!(error = %err, "cava service failed to start");
                            return;
                        }
                    };

                    // `values` is the same Property across restarts, so this
                    // stream keeps flowing after a setter restarts cava.
                    let mut values = service.values.watch();

                    loop {
                        tokio::select! {
                            next = values.next() => {
                                let Some(next) = next else { break };
                                if out.send(CavaCommandOutput::Values(next)).is_err() {
                                    break;
                                }
                            }
                            changed = config_rx.changed() => {
                                if changed.is_err() {
                                    break;
                                }
                                let desired = config_rx.borrow_and_update().clone();
                                apply_config(&service, &applied, &desired).await;
                                applied = desired;
                            }
                        }
                    }
                })
                .drop_on_shutdown()
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            CavaInput::SetConfig(config) => {
                let width_changed = self.config.borrow().bar_width != config.bar_width;
                self.config.send_if_modified(|current| {
                    if *current == config {
                        false
                    } else {
                        *current = config;
                        true
                    }
                });
                // Bar width is purely visual: resize in place, no cava restart.
                if width_changed {
                    for bar in &self.bars {
                        self.apply_bar_width(bar);
                    }
                }
            }
        }
    }

    fn update_cmd_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            CavaCommandOutput::Values(values) => {
                self.set_values(&widgets.bars, &values);
            }
        }
    }
}

async fn build_service(config: &CavaConfig) -> Result<CavaService, wayle_cava::Error> {
    CavaService::builder()
        .bars(config.bars)
        .framerate(FRAMERATE)
        .autosens(config.autosens)
        .stereo(config.stereo)
        .noise_reduction(config.noise_reduction.0)
        .monstercat(config.monstercat.0)
        .waves(config.waves)
        .low_cutoff(config.low_cutoff)
        .high_cutoff(config.high_cutoff)
        .input(INPUT)
        .source(SOURCE)
        .build()
        .await
}

/// Every `set_*` on the service restarts cava, so only call the ones whose
/// value actually changed. Setting the `Property` fields directly would update
/// the value without restarting, and the stream would keep the old behavior.
async fn apply_config(service: &CavaService, old: &CavaConfig, new: &CavaConfig) {
    macro_rules! apply {
        ($setter:ident, $field:ident $(. $inner:tt)?) => {
            if old.$field != new.$field {
                if let Err(err) = service.$setter(new.$field$(.$inner)?).await {
                    error!(error = %err, setting = stringify!($field), "failed to apply cava setting");
                }
            }
        };
    }

    apply!(set_bars, bars);
    apply!(set_autosens, autosens);
    apply!(set_stereo, stereo);
    apply!(set_noise_reduction, noise_reduction.0);
    apply!(set_monstercat, monstercat.0);
    apply!(set_waves, waves);
    apply!(set_low_cutoff, low_cutoff);
    apply!(set_high_cutoff, high_cutoff);
}

impl CavaModel {
    /// Bars move perpendicular to the direction they're stacked in.
    fn bar_orientation(&self) -> gtk::Orientation {
        match self.orientation {
            gtk::Orientation::Vertical => gtk::Orientation::Horizontal,
            _ => gtk::Orientation::Vertical,
        }
    }

    fn new_level_bar(&self, inverted: bool) -> gtk::LevelBar {
        let bar_orientation = self.bar_orientation();
        let vertical = bar_orientation == gtk::Orientation::Vertical;

        let bar = gtk::LevelBar::builder()
            .orientation(bar_orientation)
            .inverted(inverted)
            .min_value(0.0)
            .max_value(MAX_VALUE)
            .hexpand(!vertical)
            .vexpand(vertical)
            .build();

        // GTK's default offsets (low/high/full) would tag bars and style them
        // like a battery level. Drop them so the filled block is styled only by CSS.
        for name in ["low", "high", "full"] {
            bar.remove_offset_value(Some(name));
        }

        bar
    }

    /// Each value gets two level bars back to back in a homogeneous box, so each
    /// takes half the length. Both fill from the shared middle outward.
    fn new_bar(&self) -> CavaBar {
        let bar_orientation = self.bar_orientation();
        let vertical = bar_orientation == gtk::Orientation::Vertical;

        let container = gtk::Box::builder()
            .orientation(bar_orientation)
            .css_classes(["cava-bar"])
            .homogeneous(true)
            .spacing(0)
            .hexpand(!vertical)
            .vexpand(vertical)
            .build();

        // Top/left half: inverted, so it fills from its bottom/right end (the middle).
        let start = self.new_level_bar(true);
        // Bottom/right half: normal, so it fills from its top/left end (the middle).
        let end = self.new_level_bar(false);

        container.append(&start);
        container.append(&end);

        let bar = CavaBar {
            container,
            start,
            end,
        };
        self.apply_bar_width(&bar);
        bar
    }

    /// Thickness of a bar along the direction the bars are stacked.
    fn apply_bar_width(&self, bar: &CavaBar) {
        let width = self.config.borrow().bar_width as i32;
        match self.orientation {
            gtk::Orientation::Vertical => bar.container.set_height_request(width),
            _ => bar.container.set_width_request(width),
        }
    }

    fn rebuild_bars(&mut self, container: &gtk::Box, count: usize) {
        for bar in self.bars.drain(..) {
            container.remove(&bar.container);
        }
        for _ in 0..count {
            let bar = self.new_bar();
            container.append(&bar.container);
            self.bars.push(bar);
        }
    }

    fn set_values(&mut self, container: &gtk::Box, values: &[f64]) {
        if self.bars.len() != values.len() {
            self.rebuild_bars(container, values.len());
        }
        for (bar, value) in self.bars.iter().zip(values) {
            let value = value.clamp(0.0, MAX_VALUE);
            bar.start.set_value(value);
            bar.end.set_value(value);
        }
    }
}
