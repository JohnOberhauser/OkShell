use crate::menus::menu::{MenuModel, MenuOutput};
use crate::menus::menu_widgets::app_launcher::app_launcher::{
    AppLauncherInit, AppLauncherModel, AppLauncherOutput,
};
use crate::menus::menu_widgets::audio_in::audio_in_menu_widget::{
    AudioInMenuWidgetInit, AudioInMenuWidgetModel,
};
use crate::menus::menu_widgets::audio_out::audio_out_menu_widget::{
    AudioOutMenuWidgetInit, AudioOutMenuWidgetModel,
};
use crate::menus::menu_widgets::bluetooth::bluetooth_menu_widget::{
    BluetoothMenuWidgetInit, BluetoothMenuWidgetModel,
};
use crate::menus::menu_widgets::calendar::{CalendarInit, CalendarModel};
use crate::menus::menu_widgets::clipboard::clipboard::{
    ClipboardInit, ClipboardModel, ClipboardOutput,
};
use crate::menus::menu_widgets::clock::{ClockInit, ClockModel};
use crate::menus::menu_widgets::container::{ContainerInit, ContainerModel};
use crate::menus::menu_widgets::divider::{DividerMenuWidgetInit, DividerMenuWidgetModel};
use crate::menus::menu_widgets::media_player::media_players::{
    MediaPlayersInit, MediaPlayersModel,
};
use crate::menus::menu_widgets::network::network_menu_widget::{
    NetworkMenuWidgetInit, NetworkMenuWidgetModel,
};
use crate::menus::menu_widgets::notifications::notifications::{
    NotificationsInit, NotificationsModel, NotificationsOutput,
};
use crate::menus::menu_widgets::power_profile::power_profile_menu_widget::{
    PowerProfileMenuWidgetInit, PowerProfileMenuWidgetModel,
};
use crate::menus::menu_widgets::quick_action::quick_actions::{
    QuickActionsInit, QuickActionsModel, QuickActionsOutput,
};
use crate::menus::menu_widgets::screen_record::screen_record_menu_widget::{
    ScreenRecordMenuWidgetInit, ScreenRecordMenuWidgetModel, ScreenRecordMenuWidgetOutput,
};
use crate::menus::menu_widgets::screenshot::screenshot_menu_widget::{
    ScreenshotMenuWidgetInit, ScreenshotMenuWidgetModel, ScreenshotMenuWidgetOutput,
};
use crate::menus::menu_widgets::spacer::{SpacerInit, SpacerModel};
use crate::menus::menu_widgets::theme_picker::theme_picker_menu_widget::{
    ThemePickerMenuWidgetInit, ThemePickerMenuWidgetModel,
};
use crate::menus::menu_widgets::wallpaper::wallpaper_menu_widget::{
    WallpaperMenuWidgetInit, WallpaperMenuWidgetModel,
};
use crate::menus::menu_widgets::weather::weather::{WeatherInit, WeatherModel};
use okshell_common::dynamic_box::generic_widget_controller::GenericWidgetController;
use okshell_config::schema::menu_widgets::MenuWidget;
use relm4::{Component, ComponentSender, gtk};

pub fn build_widget(
    menu_widget: &MenuWidget,
    orientation: gtk::Orientation,
    sender: &ComponentSender<MenuModel>,
) -> Box<dyn GenericWidgetController> {
    match menu_widget {
        MenuWidget::AppLauncher => Box::new(
            AppLauncherModel::builder()
                .launch(AppLauncherInit {})
                .forward(sender.output_sender(), |msg| match msg {
                    AppLauncherOutput::CloseMenu => MenuOutput::CloseMenu,
                }),
        ),
        MenuWidget::AudioInput => Box::new(
            AudioInMenuWidgetModel::builder()
                .launch(AudioInMenuWidgetInit {})
                .detach(),
        ),
        MenuWidget::AudioOutput => Box::new(
            AudioOutMenuWidgetModel::builder()
                .launch(AudioOutMenuWidgetInit {})
                .detach(),
        ),
        MenuWidget::Bluetooth => Box::new(
            BluetoothMenuWidgetModel::builder()
                .launch(BluetoothMenuWidgetInit {})
                .detach(),
        ),
        MenuWidget::Calendar => Box::new(CalendarModel::builder().launch(CalendarInit {}).detach()),
        MenuWidget::Clipboard => {
            Box::new(ClipboardModel::builder().launch(ClipboardInit {}).forward(
                sender.output_sender(),
                |msg| match msg {
                    ClipboardOutput::CloseMenu => MenuOutput::CloseMenu,
                },
            ))
        }
        MenuWidget::Clock => Box::new(ClockModel::builder().launch(ClockInit {}).detach()),
        MenuWidget::Divider => Box::new(
            DividerMenuWidgetModel::builder()
                .launch(DividerMenuWidgetInit { orientation })
                .detach(),
        ),
        MenuWidget::MediaPlayer => Box::new(
            MediaPlayersModel::builder()
                .launch(MediaPlayersInit {})
                .detach(),
        ),
        MenuWidget::Network => Box::new(
            NetworkMenuWidgetModel::builder()
                .launch(NetworkMenuWidgetInit {})
                .detach(),
        ),
        MenuWidget::Notifications => Box::new(
            NotificationsModel::builder()
                .launch(NotificationsInit {})
                .forward(sender.output_sender(), |msg| match msg {
                    NotificationsOutput::CloseMenu => MenuOutput::CloseMenu,
                }),
        ),
        MenuWidget::PowerProfiles => Box::new(
            PowerProfileMenuWidgetModel::builder()
                .launch(PowerProfileMenuWidgetInit {})
                .detach(),
        ),
        MenuWidget::QuickActions(config) => Box::new(
            QuickActionsModel::builder()
                .launch(QuickActionsInit {
                    config: config.clone(),
                })
                .forward(sender.output_sender(), |msg| match msg {
                    QuickActionsOutput::CloseMenu => MenuOutput::CloseMenu,
                }),
        ),
        MenuWidget::Container(config) => Box::new(
            ContainerModel::builder()
                .launch(ContainerInit {
                    config: config.clone(),
                    menu_sender: sender.clone(),
                })
                .detach(),
        ),
        MenuWidget::Screenshots => Box::new(
            ScreenshotMenuWidgetModel::builder()
                .launch(ScreenshotMenuWidgetInit {})
                .forward(sender.output_sender(), |msg| match msg {
                    ScreenshotMenuWidgetOutput::CloseMenu => MenuOutput::CloseMenu,
                }),
        ),
        MenuWidget::ScreenRecording => Box::new(
            ScreenRecordMenuWidgetModel::builder()
                .launch(ScreenRecordMenuWidgetInit {})
                .forward(sender.output_sender(), |msg| match msg {
                    ScreenRecordMenuWidgetOutput::CloseMenu => MenuOutput::CloseMenu,
                }),
        ),
        MenuWidget::Spacer(config) => Box::new(
            SpacerModel::builder()
                .launch(SpacerInit {
                    config: config.clone(),
                    orientation,
                })
                .detach(),
        ),
        MenuWidget::ThemePicker => Box::new(
            ThemePickerMenuWidgetModel::builder()
                .launch(ThemePickerMenuWidgetInit {})
                .detach(),
        ),
        MenuWidget::Wallpaper => Box::new(
            WallpaperMenuWidgetModel::builder()
                .launch(WallpaperMenuWidgetInit {
                    thumbnail_width: 180,
                    thumbnail_height: 120,
                    row_count: 3,
                })
                .detach(),
        ),
        MenuWidget::Weather => Box::new(WeatherModel::builder().launch(WeatherInit {}).detach()),
    }
}
