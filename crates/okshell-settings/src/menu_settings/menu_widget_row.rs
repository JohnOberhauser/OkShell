use relm4::{gtk, Component, ComponentController, Controller, FactorySender};
use relm4::factory::{DynamicIndex, FactoryComponent};
use relm4::gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use okshell_config::schema::menu_widgets::{ContainerConfig, MenuWidget, QuickActionWidget, QuickActionsConfig, SpacerConfig};
use okshell_config::schema::position::Orientation;
use crate::menu_settings::container::{ContainerConfigModel, ContainerConfigOutput};
use crate::menu_settings::quick_actions_list::{QuickActionListModel, QuickActionListOutput};
use crate::menu_settings::spacer::{SpacerConfigModel, SpacerConfigOutput};

#[derive(Debug)]
pub struct MenuWidgetRowModel {
    pub widget: MenuWidget,
    pub index: DynamicIndex,
    pub container_list: Option<Controller<ContainerConfigModel>>,
    pub quick_actions_list: Option<Controller<QuickActionListModel>>,
    pub spacer_config: Option<Controller<SpacerConfigModel>>,
}

#[derive(Debug)]
pub enum MenuWidgetRowInput {
    // Config changes for Spacer
    SpacerHeightChanged(i32),
    // Config changes for Container
    ContainerOrientationChanged(Orientation),
    ContainerSpacingChanged(i32),
    ContainerMinWidthChanged(i32),
    ContainerWidgetsChanged(Vec<MenuWidget>),
    // Config changes for QuickActions
    QuickActionsChanged(Vec<QuickActionWidget>),
}

#[derive(Debug)]
pub enum MenuWidgetRowOutput {
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
    Remove(DynamicIndex),
    /// The widget at this index changed its internal config
    WidgetChanged(DynamicIndex, MenuWidget),
}

#[relm4::factory(pub)]
impl FactoryComponent for MenuWidgetRowModel {
    type Init = MenuWidget;
    type Input = MenuWidgetRowInput;
    type Output = MenuWidgetRowOutput;
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
                    set_hexpand: true,
                    set_halign: gtk::Align::Start,
                    add_css_class: "label-small",
                    #[watch]
                    set_label: self.widget.display_name(),
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_icon_name: "menu-up-symbolic",
                    connect_clicked[sender, index] => move |_| {
                        sender.output(MenuWidgetRowOutput::MoveUp(index.clone())).unwrap();
                    },
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_icon_name: "menu-down-symbolic",
                    connect_clicked[sender, index] => move |_| {
                        sender.output(MenuWidgetRowOutput::MoveDown(index.clone())).unwrap();
                    },
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_icon_name: "close-symbolic",
                    connect_clicked[sender, index] => move |_| {
                        sender.output(MenuWidgetRowOutput::Remove(index.clone())).unwrap();
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

    fn init_model(
        widget: Self::Init,
        index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self {
            widget,
            index: index.clone(),
            container_list: None,
            quick_actions_list: None,
            spacer_config: None,
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

        self.build_config(&widgets.config_area, index, &sender);

        widgets
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            MenuWidgetRowInput::SpacerHeightChanged(h) => {
                if let MenuWidget::Spacer(ref mut config) = self.widget {
                    config.size = h;
                }
                sender.output(MenuWidgetRowOutput::WidgetChanged(
                    self.index.clone(),
                    self.widget.clone(),
                )).unwrap();
            }
            MenuWidgetRowInput::ContainerOrientationChanged(o) => {
                if let MenuWidget::Container(ref mut config) = self.widget {
                    config.orientation = o;
                }
                sender.output(MenuWidgetRowOutput::WidgetChanged(
                    self.index.clone(),
                    self.widget.clone(),
                )).unwrap();
            }
            MenuWidgetRowInput::ContainerSpacingChanged(s) => {
                if let MenuWidget::Container(ref mut config) = self.widget {
                    config.spacing = s;
                }
                sender.output(MenuWidgetRowOutput::WidgetChanged(
                    self.index.clone(),
                    self.widget.clone(),
                )).unwrap();
            }
            MenuWidgetRowInput::ContainerMinWidthChanged(w) => {
                if let MenuWidget::Container(ref mut config) = self.widget {
                    config.minimum_width = w;
                }
                sender.output(MenuWidgetRowOutput::WidgetChanged(
                    self.index.clone(),
                    self.widget.clone(),
                )).unwrap();
            }
            MenuWidgetRowInput::ContainerWidgetsChanged(w) => {
                if let MenuWidget::Container(ref mut config) = self.widget {
                    config.widgets = w;
                }
                sender.output(MenuWidgetRowOutput::WidgetChanged(
                    self.index.clone(),
                    self.widget.clone(),
                )).unwrap();
            }
            MenuWidgetRowInput::QuickActionsChanged(actions) => {
                if let MenuWidget::QuickActions(ref mut config) = self.widget {
                    config.widgets = actions;
                }
                sender.output(MenuWidgetRowOutput::WidgetChanged(
                    self.index.clone(),
                    self.widget.clone(),
                )).unwrap();
            }
        }
    }
}

impl MenuWidgetRowModel {
    fn has_config(&self) -> bool {
        matches!(
            self.widget,
            MenuWidget::Container(_) | MenuWidget::Spacer(_) | MenuWidget::QuickActions(_)
        )
    }

    fn build_config(
        &mut self,
        config_area: &gtk::Box,
        _index: &DynamicIndex,
        sender: &FactorySender<Self>,
    ) {
        let widget = self.widget.clone();
        match widget {
            MenuWidget::Spacer(config) => {
                self.build_spacer_config(config_area, &config, sender);
            }
            MenuWidget::Container(config) => {
                self.build_container_config(config_area, &config, sender);
            }
            MenuWidget::QuickActions(config) => {
                self.build_quick_actions_config(config_area, &config, sender);
            }
            _ => {}
        }
    }

    fn build_spacer_config(
        &mut self,
        config_area: &gtk::Box,
        config: &SpacerConfig,
        sender: &FactorySender<Self>,
    ) {
        let sender_clone = sender.clone();
        let spacer = SpacerConfigModel::builder()
            .launch(config.clone())
            .forward(sender_clone.input_sender(), |output| match output {
                SpacerConfigOutput::SizeChanged(s) => MenuWidgetRowInput::SpacerHeightChanged(s),
            });

        config_area.append(spacer.widget());
        self.spacer_config = Some(spacer);
    }

    fn build_container_config(
        &mut self,
        config_area: &gtk::Box,
        config: &ContainerConfig,
        sender: &FactorySender<Self>,
    ) {
        let sender_clone = sender.clone();
        let container = ContainerConfigModel::builder()
            .launch(config.clone())
            .forward(sender_clone.input_sender(), |output| match output {
                ContainerConfigOutput::SpacingChanged(s) => MenuWidgetRowInput::ContainerSpacingChanged(s),
                ContainerConfigOutput::MinWidthChanged(w) => MenuWidgetRowInput::ContainerMinWidthChanged(w),
                ContainerConfigOutput::OrientationChanged(o) => MenuWidgetRowInput::ContainerOrientationChanged(o),
                ContainerConfigOutput::WidgetsChanged(w) => MenuWidgetRowInput::ContainerWidgetsChanged(w),
            });

        config_area.append(container.widget());
        self.container_list = Some(container);
    }

    fn build_quick_actions_config(
        &mut self,
        config_area: &gtk::Box,
        config: &QuickActionsConfig,
        sender: &FactorySender<Self>,
    ) {
        let sender_clone = sender.clone();
        let actions_list = QuickActionListModel::builder()
            .launch(config.widgets.clone())
            .forward(sender_clone.input_sender(), |output| match output {
                QuickActionListOutput::Changed(actions) => {
                    MenuWidgetRowInput::QuickActionsChanged(actions)
                }
            });

        config_area.append(actions_list.widget());
        self.quick_actions_list = Some(actions_list);
    }
}