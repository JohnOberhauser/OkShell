use crate::bar_settings::cava_config::{CavaConfigModel, CavaConfigOutput};
use okshell_config::schema::bar_widgets::{BarWidget, CavaConfig};
use relm4::factory::{DynamicIndex, FactoryComponent};
use relm4::gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::{Component, ComponentController, Controller, FactorySender, gtk};

#[derive(Debug)]
pub struct ActiveWidgetModel {
    pub widget: BarWidget,
    index: DynamicIndex,
    cava_config: Option<Controller<CavaConfigModel>>,
}

#[derive(Debug)]
pub enum ActiveWidgetInput {
    CavaChanged(CavaConfig),
}

#[derive(Debug)]
pub enum ActiveWidgetOutput {
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
    Remove(DynamicIndex),
    /// The widget at this index changed its internal config
    WidgetChanged(DynamicIndex, BarWidget),
}

#[relm4::factory(pub)]
impl FactoryComponent for ActiveWidgetModel {
    type Init = BarWidget;
    type Input = ActiveWidgetInput;
    type Output = ActiveWidgetOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,

                gtk::Label {
                    add_css_class: "label-small",
                    set_hexpand: true,
                    set_halign: gtk::Align::Start,
                    #[watch]
                    set_label: self.widget.display_name(),
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_icon_name: "menu-up-symbolic",
                    connect_clicked[sender, index] => move |_| {
                        sender.output(ActiveWidgetOutput::MoveUp(index.clone())).unwrap();
                    },
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_icon_name: "menu-down-symbolic",
                    connect_clicked[sender, index] => move |_| {
                        sender.output(ActiveWidgetOutput::MoveDown(index.clone())).unwrap();
                    },
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_icon_name: "close-symbolic",
                    connect_clicked[sender, index] => move |_| {
                        sender.output(ActiveWidgetOutput::Remove(index.clone())).unwrap();
                    },
                },
            },

            // Inline config area (only shown for widgets with config)
            #[name = "config_area"]
            gtk::Box {
                add_css_class: "settings-menu-widget-section",
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 4,
                #[watch]
                set_visible: self.has_config(),
            },
        }
    }

    fn init_model(widget: Self::Init, index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            widget,
            index: index.clone(),
            cava_config: None,
        }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        self.build_config(&widgets.config_area, &sender);

        widgets
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            ActiveWidgetInput::CavaChanged(config) => {
                if let BarWidget::Cava(ref mut current) = self.widget {
                    *current = config;
                }
                sender
                    .output(ActiveWidgetOutput::WidgetChanged(
                        self.index.clone(),
                        self.widget.clone(),
                    ))
                    .unwrap();
            }
        }
    }
}

impl ActiveWidgetModel {
    fn has_config(&self) -> bool {
        matches!(self.widget, BarWidget::Cava(_))
    }

    fn build_config(&mut self, config_area: &gtk::Box, sender: &FactorySender<Self>) {
        if let BarWidget::Cava(config) = &self.widget {
            let cava = CavaConfigModel::builder().launch(config.clone()).forward(
                sender.input_sender(),
                |output| match output {
                    CavaConfigOutput::Changed(config) => ActiveWidgetInput::CavaChanged(config),
                },
            );

            config_area.append(cava.widget());
            self.cava_config = Some(cava);
        }
    }
}
