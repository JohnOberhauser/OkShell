use crate::menus::menu_widgets::app_launcher::app_launcher_item::AppLauncherItemOutput::CloseMenu;
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, IconsStoreFields, ThemeStoreFields};
use okshell_config::schema::themes::Themes;
use okshell_utils::app_icon::app_icon::set_icon;
use okshell_utils::launch::launch_detached;
use reactive_graph::traits::GetUntracked;
use relm4::gtk::gio::DesktopAppInfo;
use relm4::gtk::glib::GString;
use relm4::gtk::prelude::{
    ActionMapExt, AppInfoExt, ButtonExt, OrientableExt, PopoverExt, WidgetExt,
};
use relm4::gtk::{gio, pango};
use relm4::{Component, ComponentParts, ComponentSender, RelmWidgetExt, Sender, gtk};

#[derive(Debug, Clone)]
pub(crate) struct AppLauncherItemModel {
    pub app_info: DesktopAppInfo,
    popover: Option<gtk::PopoverMenu>,
    pub hidden: bool,
    is_selected: bool,
}

#[derive(Debug)]
pub(crate) enum AppLauncherItemInput {
    Clicked,
    RightClicked,
    HiddenChanged(bool),
    NewSelectedId(Option<GString>),
    ThemeChanged(String, Themes, bool, f64, f64, f64),
}

#[derive(Debug)]
pub(crate) enum AppLauncherItemOutput {
    CloseMenu,
    Hide(String),
    Unhide(String),
}

#[derive(Debug)]
pub(crate) enum AppLauncherItemCommandOutput {}

pub(crate) struct AppLauncherItemInit {
    pub app_info: DesktopAppInfo,
    pub(crate) hidden: bool,
}

#[relm4::component(pub)]
impl Component for AppLauncherItemModel {
    type CommandOutput = AppLauncherItemCommandOutput;
    type Input = AppLauncherItemInput;
    type Output = AppLauncherItemOutput;
    type Init = AppLauncherItemInit;

    view! {
        #[root]
        #[name = "button"]
        gtk::Button {
            #[watch]
            set_css_classes: if model.is_selected {
                &["ok-button-surface", "app-launcher-item", "selected"]
            } else {
                &["ok-button-surface", "app-launcher-item"]
            },
            set_vexpand: false,
            set_hexpand: true,
            set_can_focus: false,
            connect_clicked[sender] => move |_| {
                sender.input(AppLauncherItemInput::Clicked);
            },
            add_controller = gtk::GestureClick::builder().button(3).build() {
                connect_released[sender] => move |_, _, _, _| {
                    sender.input(AppLauncherItemInput::RightClicked);
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                #[name="image"]
                gtk::Image {
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_margin_end: 12,
                },

                #[name = "label"]
                gtk:: Label {
                    add_css_class: "label-medium-bold",
                    set_label: model.app_info.name().as_str(),
                    set_ellipsize: pango::EllipsizeMode::End,
                }
            },
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let base_config = config_manager().config();

        let model = AppLauncherItemModel {
            app_info: params.app_info,
            popover: None,
            hidden: params.hidden,
            is_selected: false,
        };

        let widgets = view_output!();

        let app_info = model.app_info.clone();
        set_icon(
            &Some(app_info),
            &None,
            &widgets.image,
            base_config.theme().icons().app_icon_theme().get_untracked(),
            &config_manager().config().theme().theme().get_untracked(),
            config_manager()
                .config()
                .theme()
                .icons()
                .apply_theme_filter()
                .get_untracked(),
            config_manager()
                .config()
                .theme()
                .icons()
                .filter_strength()
                .get_untracked()
                .get(),
            config_manager()
                .config()
                .theme()
                .icons()
                .monochrome_strength()
                .get_untracked()
                .get(),
            config_manager()
                .config()
                .theme()
                .icons()
                .contrast_strength()
                .get_untracked()
                .get(),
        );

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AppLauncherItemInput::Clicked => {
                launch_detached(&self.app_info);
                let _ = sender.output(CloseMenu);
            }
            AppLauncherItemInput::RightClicked => {
                if let Some(window) = widgets.button.toplevel_window() {
                    window.set_keyboard_mode(KeyboardMode::OnDemand);
                }

                let action_group = gio::SimpleActionGroup::new();
                let menu = gio::Menu::new();

                if let Some(id) = self.app_info.id() {
                    let id = id.to_string();
                    if self.hidden {
                        let action = gio::SimpleAction::new("unhide", None);
                        let sender = sender.clone();
                        action.connect_activate(move |_, _| {
                            let id = id.clone();
                            let _ = sender.output(AppLauncherItemOutput::Unhide(id));
                        });
                        action_group.add_action(&action);
                        menu.append(Some("Unhide"), Some("main.unhide"));
                    } else {
                        let action = gio::SimpleAction::new("hide", None);
                        let sender = sender.clone();
                        action.connect_activate(move |_, _| {
                            let id = id.clone();
                            let _ = sender.output(AppLauncherItemOutput::Hide(id));
                        });
                        action_group.add_action(&action);
                        menu.append(Some("Hide"), Some("main.hide"));
                    }
                }

                let popover = gtk::PopoverMenu::from_model(Some(&menu));
                popover.set_has_arrow(false);
                popover.insert_action_group("main", Some(&action_group));
                popover.set_parent(&widgets.button);

                let button = widgets.button.clone();
                popover.connect_closed(move |_| {
                    if let Some(window) = button.toplevel_window() {
                        window.set_keyboard_mode(KeyboardMode::Exclusive);
                    }
                });

                popover.popup();

                self.popover = Some(popover);
            }
            AppLauncherItemInput::HiddenChanged(hidden) => {
                self.hidden = hidden;
            }
            AppLauncherItemInput::NewSelectedId(selected_id) => {
                self.is_selected = self.app_info.id() == selected_id;
            }
            AppLauncherItemInput::ThemeChanged(
                theme,
                color_theme,
                apply_theme,
                filter_strength,
                monochrome_strength,
                contrast_strength,
            ) => {
                let app_info = self.app_info.clone();
                set_icon(
                    &Some(app_info),
                    &None,
                    &widgets.image,
                    theme,
                    &color_theme,
                    apply_theme,
                    filter_strength,
                    monochrome_strength,
                    contrast_strength,
                );
            }
        }

        self.update_view(widgets, sender);
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: Sender<Self::Output>) {
        if let Some(popover) = self.popover.take() {
            popover.unparent();
        }
    }
}
