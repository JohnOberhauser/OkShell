use std::cell::{Cell, RefCell};
use std::rc::Rc;

use okshell_services::sys_info_service;
use okshell_utils::sys_info::spawn_cpu_watcher;
use relm4::gtk::glib;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};

const CHART_HEIGHT: i32 = 60;
const BAR_SPACING: i32 = 4;
const ANIM_DURATION_US: f64 = 400_000.0; // 400ms

#[derive(Debug, Clone)]
pub(crate) struct CpuUsageModel {
    bars: Rc<BarsAnim>,
    temp_label: String,
    usage_percent_label: String,
    average_frequency_label: String,
    max_frequency_label: String,
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

/// One LevelBar per core plus the animation state driving them.
/// All values are 0.0..=100.0.
#[derive(Debug, Default)]
struct BarsAnim {
    bars: RefCell<Vec<gtk::LevelBar>>,
    current: RefCell<Vec<f64>>,
    from: RefCell<Vec<f64>>,
    to: RefCell<Vec<f64>>,
    start_us: Cell<i64>,
    animating: Cell<bool>,
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
            set_css_classes: &["cpu-usage-menu-widget"],
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,
            set_hexpand: true,
            set_vexpand: false,

            gtk::Label {
                add_css_class: "label-large-bold",
                set_label: "CPU",
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_homogeneous: true,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,

                    gtk::Label {
                        add_css_class: "label-medium-bold",
                        set_label: "Usage:",
                        set_margin_end: 12,
                    },

                    gtk::Label {
                        add_css_class: "label-medium",
                        #[watch]
                        set_label: model.usage_percent_label.as_str(),
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,

                    gtk::Label {
                        add_css_class: "label-medium-bold",
                        set_label: "Avg:",
                        set_margin_end: 12,
                    },

                    gtk::Label {
                        add_css_class: "label-medium",
                        #[watch]
                        set_label: model.average_frequency_label.as_str(),
                    },
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_homogeneous: true,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,

                    gtk::Label {
                        add_css_class: "label-medium-bold",
                        set_label: "Temp:",
                        set_margin_end: 12,
                    },

                    gtk::Label {
                        add_css_class: "label-medium",
                        #[watch]
                        set_label: model.temp_label.as_str(),
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,

                    gtk::Label {
                        add_css_class: "label-medium-bold",
                        set_label: "Max:",
                        set_margin_end: 12,
                    },

                    gtk::Label {
                        add_css_class: "label-medium",
                        #[watch]
                        set_label: model.max_frequency_label.as_str(),
                    },
                },
            },

            #[name = "chart"]
            gtk::Box {
                add_css_class: "cpu-core-bars",
                set_orientation: gtk::Orientation::Horizontal,
                set_homogeneous: true,
                set_hexpand: true,
                set_valign: gtk::Align::Center,
                set_spacing: BAR_SPACING,
                set_height_request: CHART_HEIGHT,
            }
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        spawn_cpu_watcher(&sender, || CpuUsageCommandOutput::CpuStateChanged);

        let model = CpuUsageModel {
            bars: Rc::new(BarsAnim::default()),
            temp_label: String::new(),
            usage_percent_label: String::new(),
            average_frequency_label: String::new(),
            max_frequency_label: String::new(),
        };

        let widgets = view_output!();

        // Build the bars and fill them right away instead of waiting for the first update.
        model.bars.animate_to(&widgets.chart, core_usages());

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
            CpuUsageCommandOutput::CpuStateChanged => {
                self.bars.animate_to(&widgets.chart, core_usages());

                let cpu = sys_info_service().cpu.get();

                if let Some(temp) = cpu.temperature_celsius {
                    self.temp_label = format!("{:.0}°C", temp);
                }
                self.usage_percent_label = format!("{:.0}%", cpu.usage_percent);
                self.average_frequency_label = format!("{} MHz", cpu.avg_frequency_mhz);
                self.max_frequency_label = format!("{} MHz", cpu.max_frequency_mhz);
            }
        }
        self.update_view(widgets, sender);
    }
}

impl BarsAnim {
    fn animate_to(self: &Rc<Self>, container: &gtk::Box, targets: Vec<f64>) {
        let targets: Vec<f64> = targets.into_iter().map(|v| v.clamp(0.0, 100.0)).collect();
        if *self.to.borrow() == targets {
            return;
        }

        {
            let mut current = self.current.borrow_mut();
            if current.len() != targets.len() {
                // Core count changed (or first update): rebuild the bars.
                current.resize(targets.len(), 0.0);
                self.rebuild_bars(container, targets.len());
            }
            // Start from wherever the bars are right now, even mid-animation.
            *self.from.borrow_mut() = current.clone();
        }
        *self.to.borrow_mut() = targets;
        self.start_us.set(glib::monotonic_time());

        if self.animating.replace(true) {
            return; // tick callback already running; it'll pick up the new targets
        }

        let anim = self.clone();
        container.add_tick_callback(move |_, clock| {
            let elapsed = (clock.frame_time() - anim.start_us.get()) as f64;
            let t = (elapsed / ANIM_DURATION_US).clamp(0.0, 1.0);
            let eased = 1.0 - (1.0 - t).powi(3); // ease-out cubic

            {
                let from = anim.from.borrow();
                let to = anim.to.borrow();
                let mut current = anim.current.borrow_mut();
                for ((cur, start), target) in current.iter_mut().zip(from.iter()).zip(to.iter()) {
                    *cur = start + (target - start) * eased;
                }
            }
            anim.apply();

            if t >= 1.0 {
                anim.animating.set(false);
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }

    fn rebuild_bars(&self, container: &gtk::Box, count: usize) {
        let mut bars = self.bars.borrow_mut();
        for bar in bars.drain(..) {
            container.remove(&bar);
        }
        for _ in 0..count {
            let bar = new_core_bar();
            container.append(&bar);
            bars.push(bar);
        }
    }

    fn apply(&self) {
        let bars = self.bars.borrow();
        let current = self.current.borrow();
        for (bar, value) in bars.iter().zip(current.iter()) {
            bar.set_value(*value);
        }
    }
}

fn new_core_bar() -> gtk::LevelBar {
    let bar = gtk::LevelBar::builder()
        .orientation(gtk::Orientation::Vertical)
        .inverted(true) // fill from the bottom up
        .min_value(0.0)
        .max_value(100.0)
        .hexpand(true)
        .vexpand(true)
        .build();

    // GTK's default offsets (low/high/full) are meant for 0..1 ranges like
    // battery levels and would tag every bar as "full". Drop them so the
    // filled block is styled only by CSS.
    for name in ["low", "high", "full"] {
        bar.remove_offset_value(Some(name));
    }

    bar
}

fn core_usages() -> Vec<f64> {
    sys_info_service()
        .cpu
        .get()
        .cores
        .iter()
        .map(|core| core.usage_percent as f64)
        .collect()
}
