use crate::menus::menu_widgets::quick_action::actions::airplane_mode::{
    AirplaneModeInit, AirplaneModeModel,
};
use crate::menus::menu_widgets::quick_action::actions::do_not_disturb::{
    DoNotDisturbInit, DoNotDisturbModel,
};
use crate::menus::menu_widgets::quick_action::actions::hypr_picker::{
    HyprPickerInit, HyprPickerModel, HyprPickerOutput,
};
use crate::menus::menu_widgets::quick_action::actions::idle_inhibitor::{
    IdleInhibitorInit, IdleInhibitorModel,
};
use crate::menus::menu_widgets::quick_action::actions::lock::{LockInit, LockModel, LockOutput};
use crate::menus::menu_widgets::quick_action::actions::logout::{LogoutInit, LogoutModel};
use crate::menus::menu_widgets::quick_action::actions::night_light::{
    NightLightInit, NightLightModel,
};
use crate::menus::menu_widgets::quick_action::actions::reboot::{RebootInit, RebootModel};
use crate::menus::menu_widgets::quick_action::actions::settings::{
    SettingsInit, SettingsModel, SettingsOutput,
};
use crate::menus::menu_widgets::quick_action::actions::shutdown::{ShutdownInit, ShutdownModel};
use okshell_common::dynamic_box::generic_widget_controller::GenericWidgetController;
use okshell_config::schema::menu_widgets::{QuickActionWidget, QuickActionsConfig};
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, RelmWidgetExt, gtk};

pub(crate) struct QuickActionsModel {
    _widget_controllers: Vec<Box<dyn GenericWidgetController>>,
}

#[derive(Debug)]
pub(crate) enum QuickActionsInput {}

#[derive(Debug)]
pub(crate) enum QuickActionsOutput {
    CloseMenu,
}

pub(crate) struct QuickActionsInit {
    pub config: QuickActionsConfig,
}

#[derive(Debug)]
pub(crate) enum QuickActionsCommandOutput {}

#[relm4::component(pub)]
impl Component for QuickActionsModel {
    type CommandOutput = QuickActionsCommandOutput;
    type Input = QuickActionsInput;
    type Output = QuickActionsOutput;
    type Init = QuickActionsInit;

    view! {
        #[root]
        #[name = "quick_actions_container"]
        gtk::Box {
            add_css_class: "quick-actions-menu-widget",
            set_orientation: gtk::Orientation::Horizontal,
            set_align: gtk::Align::Center,
            set_spacing: 12,
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut widget_controllers: Vec<Box<dyn GenericWidgetController>> = Vec::new();

        let widgets = view_output!();

        params.config.widgets.iter().for_each(|widget| {
            let controller = Self::build_widget(widget, &sender);
            widgets
                .quick_actions_container
                .append(&controller.root_widget());
            widget_controllers.push(controller);
        });

        let model = QuickActionsModel {
            _widget_controllers: widget_controllers,
        };

        ComponentParts { model, widgets }
    }
}

impl QuickActionsModel {
    fn build_widget(
        widget: &QuickActionWidget,
        sender: &ComponentSender<Self>,
    ) -> Box<dyn GenericWidgetController> {
        match widget {
            QuickActionWidget::AirplaneMode => Box::new(
                AirplaneModeModel::builder()
                    .launch(AirplaneModeInit {})
                    .detach(),
            ),
            QuickActionWidget::DoNotDisturb => Box::new(
                DoNotDisturbModel::builder()
                    .launch(DoNotDisturbInit {})
                    .detach(),
            ),
            QuickActionWidget::HyprPicker => Box::new(
                HyprPickerModel::builder()
                    .launch(HyprPickerInit {})
                    .forward(sender.output_sender(), |msg| match msg {
                        HyprPickerOutput::CloseMenu => QuickActionsOutput::CloseMenu,
                    }),
            ),
            QuickActionWidget::IdleInhibitor => Box::new(
                IdleInhibitorModel::builder()
                    .launch(IdleInhibitorInit {})
                    .detach(),
            ),
            QuickActionWidget::Lock => Box::new(LockModel::builder().launch(LockInit {}).forward(
                sender.output_sender(),
                |msg| match msg {
                    LockOutput::CloseMenu => QuickActionsOutput::CloseMenu,
                },
            )),
            QuickActionWidget::Logout => {
                Box::new(LogoutModel::builder().launch(LogoutInit {}).detach())
            }
            QuickActionWidget::Nightlight => Box::new(
                NightLightModel::builder()
                    .launch(NightLightInit {})
                    .detach(),
            ),
            QuickActionWidget::Reboot => {
                Box::new(RebootModel::builder().launch(RebootInit {}).detach())
            }
            QuickActionWidget::Settings => {
                Box::new(SettingsModel::builder().launch(SettingsInit {}).forward(
                    sender.output_sender(),
                    |msg| match msg {
                        SettingsOutput::CloseMenu => QuickActionsOutput::CloseMenu,
                    },
                ))
            }
            QuickActionWidget::Shutdown => {
                Box::new(ShutdownModel::builder().launch(ShutdownInit {}).detach())
            }
        }
    }
}
