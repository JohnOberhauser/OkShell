use okshell_config::schema::bar_widgets::CavaConfig;
use okshell_config::schema::location_query::OrdF64;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};

#[derive(Debug)]
pub struct CavaConfigModel {
    config: CavaConfig,
}

#[derive(Debug)]
pub enum CavaConfigInput {
    BarsChanged(u16),
    BarWidthChanged(u32),
    AutosensChanged(bool),
    StereoChanged(bool),
    NoiseReductionChanged(f64),
    MonstercatChanged(f64),
    WavesChanged(u32),
    LowCutoffChanged(u32),
    HighCutoffChanged(u32),
}

#[derive(Debug)]
pub enum CavaConfigOutput {
    Changed(CavaConfig),
}

#[relm4::component(pub)]
impl Component for CavaConfigModel {
    type CommandOutput = ();
    type Input = CavaConfigInput;
    type Output = CavaConfigOutput;
    type Init = CavaConfig;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 4,
            add_css_class: "cava-config",

            // Bars
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,

                gtk::Label {
                    add_css_class: "label-small",
                    set_label: "Bars",
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                gtk::SpinButton {
                    set_range: (1.0, 256.0),
                    set_increments: (1.0, 8.0),
                    set_value: model.config.bars as f64,
                    connect_value_changed[sender] => move |s| {
                        sender.input(CavaConfigInput::BarsChanged(s.value() as u16));
                    },
                },
            },

            // Bar width
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,

                gtk::Label {
                    add_css_class: "label-small",
                    set_label: "Bar width (px)",
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                gtk::SpinButton {
                    set_range: (1.0, 50.0),
                    set_increments: (1.0, 5.0),
                    set_value: model.config.bar_width as f64,
                    connect_value_changed[sender] => move |s| {
                        sender.input(CavaConfigInput::BarWidthChanged(s.value() as u32));
                    },
                },
            },

            // Autosens
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,

                gtk::Label {
                    add_css_class: "label-small",
                    set_label: "Auto sensitivity",
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                gtk::Switch {
                    set_valign: gtk::Align::Center,
                    set_active: model.config.autosens,
                    connect_active_notify[sender] => move |s| {
                        sender.input(CavaConfigInput::AutosensChanged(s.is_active()));
                    },
                },
            },

            // Stereo
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,

                gtk::Label {
                    add_css_class: "label-small",
                    set_label: "Stereo",
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                gtk::Switch {
                    set_valign: gtk::Align::Center,
                    set_active: model.config.stereo,
                    connect_active_notify[sender] => move |s| {
                        sender.input(CavaConfigInput::StereoChanged(s.is_active()));
                    },
                },
            },

            // Noise reduction
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,

                gtk::Label {
                    add_css_class: "label-small",
                    set_label: "Noise reduction",
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                gtk::SpinButton {
                    set_range: (0.0, 1.0),
                    set_increments: (0.01, 0.1),
                    set_digits: 2,
                    set_value: model.config.noise_reduction.0,
                    connect_value_changed[sender] => move |s| {
                        sender.input(CavaConfigInput::NoiseReductionChanged(s.value()));
                    },
                },
            },

            // Monstercat
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,

                gtk::Label {
                    add_css_class: "label-small",
                    set_label: "Monstercat",
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                gtk::SpinButton {
                    set_range: (0.0, 10.0),
                    set_increments: (0.1, 1.0),
                    set_digits: 2,
                    set_value: model.config.monstercat.0,
                    connect_value_changed[sender] => move |s| {
                        sender.input(CavaConfigInput::MonstercatChanged(s.value()));
                    },
                },
            },

            // Waves
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,

                gtk::Label {
                    add_css_class: "label-small",
                    set_label: "Waves",
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                gtk::SpinButton {
                    set_range: (0.0, 100.0),
                    set_increments: (1.0, 10.0),
                    set_value: model.config.waves as f64,
                    connect_value_changed[sender] => move |s| {
                        sender.input(CavaConfigInput::WavesChanged(s.value() as u32));
                    },
                },
            },

            // Low cutoff
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,

                gtk::Label {
                    add_css_class: "label-small",
                    set_label: "Low cutoff (Hz)",
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                gtk::SpinButton {
                    set_range: (1.0, 20000.0),
                    set_increments: (10.0, 100.0),
                    set_value: model.config.low_cutoff as f64,
                    connect_value_changed[sender] => move |s| {
                        sender.input(CavaConfigInput::LowCutoffChanged(s.value() as u32));
                    },
                },
            },

            // High cutoff
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,

                gtk::Label {
                    add_css_class: "label-small",
                    set_label: "High cutoff (Hz)",
                    set_halign: gtk::Align::Start,
                    set_hexpand: true,
                },

                gtk::SpinButton {
                    set_range: (1.0, 20000.0),
                    set_increments: (10.0, 100.0),
                    set_value: model.config.high_cutoff as f64,
                    connect_value_changed[sender] => move |s| {
                        sender.input(CavaConfigInput::HighCutoffChanged(s.value() as u32));
                    },
                },
            },
        }
    }

    fn init(
        config: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = CavaConfigModel { config };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        let config = &mut self.config;
        match message {
            CavaConfigInput::BarsChanged(v) => config.bars = v,
            CavaConfigInput::BarWidthChanged(v) => config.bar_width = v,
            CavaConfigInput::AutosensChanged(v) => config.autosens = v,
            CavaConfigInput::StereoChanged(v) => config.stereo = v,
            CavaConfigInput::NoiseReductionChanged(v) => config.noise_reduction = OrdF64(v),
            CavaConfigInput::MonstercatChanged(v) => config.monstercat = OrdF64(v),
            CavaConfigInput::WavesChanged(v) => config.waves = v,
            CavaConfigInput::LowCutoffChanged(v) => config.low_cutoff = v,
            CavaConfigInput::HighCutoffChanged(v) => config.high_cutoff = v,
        }
        let _ = sender.output(CavaConfigOutput::Changed(self.config.clone()));
    }
}
