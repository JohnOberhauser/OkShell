use std::cell::Cell;
use std::rc::Rc;

use okshell_services::sys_info_service;
use okshell_utils::sys_info::spawn_memory_watcher;
use relm4::gtk::glib;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};

const ANIM_DURATION_US: f64 = 400_000.0; // 400ms
const BYTES_PER_GB: f64 = 1024.0 * 1024.0 * 1024.0;

#[derive(Debug, Clone)]
pub(crate) struct MemoryUsageModel {
    used_bar: Rc<LevelAnim>,
    available_bar: Rc<LevelAnim>,
    total_label: String,
    used_label: String,
    available_label: String,
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

/// A horizontal LevelBar whose value eases to each new target.
/// Values are 0.0..=100.0.
#[derive(Debug)]
struct LevelAnim {
    bar: gtk::LevelBar,
    current: Cell<f64>,
    from: Cell<f64>,
    to: Cell<f64>,
    start_us: Cell<i64>,
    animating: Cell<bool>,
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
                },

                #[local_ref]
                used_bar -> gtk::LevelBar {
                    add_css_class: "memory-used-bar",
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_spacing: 4,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,

                    gtk::Label {
                        add_css_class: "label-medium-bold",
                        set_label: "Available:",
                        set_margin_end: 12,
                    },

                    gtk::Label {
                        add_css_class: "label-medium",
                        #[watch]
                        set_label: model.available_label.as_str(),
                    },
                },

                #[local_ref]
                available_bar -> gtk::LevelBar {
                    add_css_class: "memory-available-bar",
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
            available_bar: LevelAnim::new(),
            total_label: String::new(),
            used_label: String::new(),
            available_label: String::new(),
        };

        // Fill labels and bars right away instead of waiting for the first update.
        model.refresh();

        let used_bar = model.used_bar.widget();
        let available_bar = model.available_bar.widget();
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
        let available = memory.available_bytes as f64;

        self.total_label = format!("{:.0} GB", total / BYTES_PER_GB);
        self.used_label = format!("{:.1} GB", used / BYTES_PER_GB);
        self.available_label = format!("{:.1} GB", available / BYTES_PER_GB);

        self.used_bar.animate_to(percent_of(used, total));
        self.available_bar.animate_to(percent_of(available, total));
    }
}

fn percent_of(value: f64, total: f64) -> f64 {
    if total > 0.0 {
        value / total * 100.0
    } else {
        0.0
    }
}

impl LevelAnim {
    fn new() -> Rc<Self> {
        let bar = gtk::LevelBar::builder()
            .orientation(gtk::Orientation::Horizontal)
            .min_value(0.0)
            .max_value(100.0)
            .hexpand(true)
            .build();

        // GTK's default offsets (low/high/full) are meant for 0..1 ranges like
        // battery levels and would tag the bar as "full". Drop them so the
        // filled block is styled only by CSS.
        for name in ["low", "high", "full"] {
            bar.remove_offset_value(Some(name));
        }

        Rc::new(Self {
            bar,
            current: Cell::new(0.0),
            from: Cell::new(0.0),
            to: Cell::new(0.0),
            start_us: Cell::new(0),
            animating: Cell::new(false),
        })
    }

    fn widget(&self) -> &gtk::LevelBar {
        &self.bar
    }

    fn animate_to(self: &Rc<Self>, target: f64) {
        let target = target.clamp(0.0, 100.0);
        if (target - self.to.get()).abs() < f64::EPSILON {
            return;
        }

        // Start from wherever the bar is right now, even mid-animation.
        self.from.set(self.current.get());
        self.to.set(target);
        self.start_us.set(glib::monotonic_time());

        if self.animating.replace(true) {
            return; // tick callback already running; it'll pick up the new target
        }

        let anim = self.clone();
        self.bar.add_tick_callback(move |bar, clock| {
            let elapsed = (clock.frame_time() - anim.start_us.get()) as f64;
            let t = (elapsed / ANIM_DURATION_US).clamp(0.0, 1.0);
            let eased = 1.0 - (1.0 - t).powi(3); // ease-out cubic

            let from = anim.from.get();
            let value = from + (anim.to.get() - from) * eased;
            anim.current.set(value);
            bar.set_value(value);

            if t >= 1.0 {
                anim.animating.set(false);
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });
    }
}
