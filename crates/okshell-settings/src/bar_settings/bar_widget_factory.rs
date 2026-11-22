use crate::bar_settings::cava_config::{CavaConfigModel, CavaConfigOutput};
use crate::bar_settings::hyprland_workspaces_config::{
    HyprlandWorkspacesConfigModel, HyprlandWorkspacesConfigOutput,
};
use okshell_config::schema::bar_widgets::{BarWidget, CavaConfig, HyprlandWorkspacesConfig};
use relm4::factory::{DynamicIndex, FactoryComponent};
use relm4::gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::{Component, ComponentController, Controller, FactorySender, gtk};

#[derive(Debug)]
pub struct ActiveWidgetModel {
    pub widget: BarWidget,
    index: DynamicIndex,
    config_controller: Option<ConfigController>,
}

/// Keeps the inline config component alive for the lifetime of the row.
#[derive(Debug)]
enum ConfigController {
    Cava(Controller<CavaConfigModel>),
    HyprlandWorkspaces(Controller<HyprlandWorkspacesConfigModel>),
}

#[derive(Debug)]
pub enum ActiveWidgetInput {
    CavaChanged(CavaConfig),
    HyprlandWorkspacesChanged(HyprlandWorkspacesConfig),
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
                set_visible: self.config_controller.is_some(),
            },
        }
    }

    fn init_model(widget: Self::Init, index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            widget,
            index: index.clone(),
            config_controller: None,
        }
    }

    fn init_widgets(
        &mut self,
        index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as relm4::factory::FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        // Build config first so the #[watch]ed visibility is right on first draw.
        self.config_controller = self.build_config(&sender);

        let widgets = view_output!();

        match &self.config_controller {
            Some(ConfigController::Cava(c)) => widgets.config_area.append(c.widget()),
            Some(ConfigController::HyprlandWorkspaces(c)) => widgets.config_area.append(c.widget()),
            None => {}
        }

        widgets
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            ActiveWidgetInput::CavaChanged(config) => {
                if let BarWidget::Cava(current) = &mut self.widget {
                    *current = config;
                }
            }
            ActiveWidgetInput::HyprlandWorkspacesChanged(config) => {
                if let BarWidget::HyprlandWorkspaces(current) = &mut self.widget {
                    *current = config;
                }
            }
        }
        sender
            .output(ActiveWidgetOutput::WidgetChanged(
                self.index.clone(),
                self.widget.clone(),
            ))
            .unwrap();
    }
}

impl ActiveWidgetModel {
    fn build_config(&self, sender: &FactorySender<Self>) -> Option<ConfigController> {
        match &self.widget {
            BarWidget::Cava(config) => Some(ConfigController::Cava(
                CavaConfigModel::builder().launch(config.clone()).forward(
                    sender.input_sender(),
                    |output| match output {
                        CavaConfigOutput::Changed(c) => ActiveWidgetInput::CavaChanged(c),
                    },
                ),
            )),
            BarWidget::HyprlandWorkspaces(config) => Some(ConfigController::HyprlandWorkspaces(
                HyprlandWorkspacesConfigModel::builder()
                    .launch(config.clone())
                    .forward(sender.input_sender(), |output| match output {
                        HyprlandWorkspacesConfigOutput::Changed(c) => {
                            ActiveWidgetInput::HyprlandWorkspacesChanged(c)
                        }
                    }),
            )),
            _ => None,
        }
    }
}
