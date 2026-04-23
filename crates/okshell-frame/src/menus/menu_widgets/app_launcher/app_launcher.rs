use gtk4_layer_shell::{KeyboardMode, LayerShell};
use reactive_graph::traits::*;
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt};
use relm4::gtk::gio::DesktopAppInfo;
use relm4::gtk::prelude::*;
use relm4::gtk::{gdk, gio, RevealerTransitionType, ScrolledWindow};
use relm4::gtk::glib;
use tracing::info;
use okshell_cache::hidden_apps::{hidden_apps_store, hide_app, is_hidden, unhide_app, HiddenAppsStateStoreFields};
use okshell_common::scoped_effects::EffectScope;
use okshell_common::dynamic_box::dynamic_box::{DynamicBoxFactory, DynamicBoxInit, DynamicBoxInput, DynamicBoxModel};
use okshell_common::dynamic_box::generic_widget_controller::{
    GenericWidgetController,
    GenericWidgetControllerExtSafe,
};
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, IconsStoreFields, ThemeStoreFields};
use crate::menus::menu_widgets::app_launcher::app_launcher_item::{AppLauncherItemInit, AppLauncherItemInput, AppLauncherItemModel, AppLauncherItemOutput};
use okshell_utils::launch::launch_detached;

struct AppItem {
    app_info: DesktopAppInfo,
    hidden: bool,
}

pub(crate) struct AppLauncherModel {
    dynamic_box: Controller<DynamicBoxModel<AppItem, String>>,
    filter: String,
    apps: Vec<DesktopAppInfo>,
    filtered_list: Vec<DesktopAppInfo>,
    selected_app: Option<DesktopAppInfo>,
    show_hidden_apps: bool,
    is_revealed: bool,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum AppLauncherInput {
    UpdateAppsList,
    FilterChanged(String),
    Activate,
    ParentRevealChanged(bool),
    DownPressed,
    UpPressed,
    HiddenAppsChanged,
    HideApp(String),
    UnhideApp(String),
    CloseMenu,
    ShowHiddenAppsChanged,
    ThemeChanged,
}

#[derive(Debug)]
pub(crate) enum AppLauncherOutput {
    CloseMenu,
}

pub(crate) struct AppLauncherInit {
}

#[derive(Debug)]
pub(crate) enum AppLauncherCommandOutput {}

#[relm4::component(pub)]
impl Component for AppLauncherModel {
    type CommandOutput = AppLauncherCommandOutput;
    type Input = AppLauncherInput;
    type Output = AppLauncherOutput;
    type Init = AppLauncherInit;

    view! {
        #[root]
        #[name = "root"]
        gtk::Box {
            add_css_class: "app-launcher-menu-widget",
            set_orientation: gtk::Orientation::Vertical,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_margin_bottom: 8,

                gtk::Image {
                    add_css_class: "app-launcher-search-icon",
                    set_icon_name: Some("system-search-symbolic"),
                },

                #[name = "search_entry"]
                gtk::Entry {
                    add_css_class: "ok-entry",
                    set_placeholder_text: Some("Search"),
                    set_hexpand: true,
                    connect_changed[sender] => move |entry| {
                        sender.input(AppLauncherInput::FilterChanged(entry.text().to_string()));
                    },
                    connect_activate[sender] => move |_| {
                        sender.input(AppLauncherInput::Activate);
                    },
                },

                gtk::Button {
                    add_css_class: "ok-button-surface",
                    set_hexpand: false,
                    set_vexpand: false,
                    connect_clicked[sender] => move |_| {
                        sender.input(AppLauncherInput::ShowHiddenAppsChanged);
                    },

                    #[name="image"]
                    gtk::Image {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        #[watch]
                        set_icon_name: if model.show_hidden_apps {
                            Some("eye-symbolic")
                        } else {
                            Some("eye-off-symbolic")
                        },
                    }
                }
            },

            #[name = "scrolled_window"]
            ScrolledWindow {
                set_vscrollbar_policy: gtk::PolicyType::Automatic,
                set_propagate_natural_height: true,

                #[name = "apps_box"]
                gtk::Box {},
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let sender_clone = sender.clone();
        let monitor = gio::AppInfoMonitor::get();
        monitor.connect_changed(move |_| {
            sender_clone.input(AppLauncherInput::UpdateAppsList);
        });

        sender.input(AppLauncherInput::UpdateAppsList);
        sender.input(AppLauncherInput::FilterChanged("".to_string()));

        let sender_clone = sender.clone();
        let factory = DynamicBoxFactory::<AppItem, String> {
            id: Box::new(|item| item.app_info.id()
                .map(|s| s.to_string())
                .or_else(|| item.app_info.filename().map(|p| p.to_string_lossy().into_owned()))
                .unwrap_or_else(|| item.app_info.name().to_string())),
            create: Box::new(move |item| {
                let controller: Controller<AppLauncherItemModel> =
                    AppLauncherItemModel::builder()
                        .launch(AppLauncherItemInit {
                            app_info: item.app_info.clone(),
                            hidden: item.hidden,
                        })
                        .forward(sender_clone.input_sender(), move |msg| {
                            match msg {
                                AppLauncherItemOutput::CloseMenu => AppLauncherInput::CloseMenu,
                                AppLauncherItemOutput::Hide(id) => AppLauncherInput::HideApp(id),
                                AppLauncherItemOutput::Unhide(id) => AppLauncherInput::UnhideApp(id),
                            }
                        });
                Box::new(controller) as Box<dyn GenericWidgetController>
            }),
            update: None,
        };

        let dynamic: Controller<DynamicBoxModel<AppItem, String>> =
            DynamicBoxModel::builder()
                .launch(DynamicBoxInit{
                    factory,
                    orientation: gtk::Orientation::Vertical,
                    spacing: 10,
                    transition_type: RevealerTransitionType::SlideDown,
                    transition_duration_ms: 0,
                    reverse: false,
                    retain_entries: true,
                    allow_drag_and_drop: false,
                })
                .detach();

        let key_controller = gtk::EventControllerKey::new();
        let sender_clone = sender.clone();
        key_controller.connect_key_pressed(move |_, key, _, _| {
            match key {
                gdk::Key::Up => {
                    sender_clone.input(AppLauncherInput::UpPressed);
                    glib::Propagation::Stop
                }
                gdk::Key::Down => {
                    sender_clone.input(AppLauncherInput::DownPressed);
                    glib::Propagation::Stop
                }
                gdk::Key::Escape => {
                    let _ = sender_clone.output(AppLauncherOutput::CloseMenu);
                    glib::Propagation::Stop
                }
                _ => glib::Propagation::Proceed
            }
        });

        let mut effect_scope = EffectScope::new();

        let sender_clone = sender.clone();
        effect_scope.push(move |_| {
            let store = hidden_apps_store();
            let _ = store.apps().get();
            sender_clone.input(AppLauncherInput::HiddenAppsChanged);
        });

        let sender_clone = sender.clone();
        effect_scope.push(move |_| {
            let _ = config_manager().config().theme().icons().app_icon_theme().get();
            let _ = config_manager().config().theme().icons().apply_theme_filter().get();
            let _ = config_manager().config().theme().theme().get();
            let _ = config_manager().config().theme().icons().filter_strength().get();
            let _ = config_manager().config().theme().icons().monochrome_strength().get();
            let _ = config_manager().config().theme().icons().contrast_strength().get();
            sender_clone.input(AppLauncherInput::ThemeChanged);
        });

        let model = AppLauncherModel {
            dynamic_box: dynamic,
            filter: "".to_string(),
            apps: Vec::new(),
            filtered_list: Vec::new(),
            selected_app: None,
            show_hidden_apps: false,
            is_revealed: false,
            _effects: effect_scope,
        };

        let widgets = view_output!();

        widgets.apps_box.append(model.dynamic_box.widget());
        widgets.root.add_controller(key_controller);

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
            AppLauncherInput::UpdateAppsList => {
                let mut apps: Vec<DesktopAppInfo> = gio::AppInfo::all()
                    .into_iter()
                    .filter_map(|info| info.downcast::<DesktopAppInfo>().ok())
                    .filter(|info| !info.is_hidden() && !info.is_nodisplay())
                    .collect();

                apps.sort_by_key(|info| info.display_name().to_lowercase());

                self.apps = apps;
            }
            AppLauncherInput::FilterChanged(filter) => {
                self.filter = filter;
                let filter = self.filter.to_lowercase();
                let apps = self.apps.clone();
                let show_hidden = self.show_hidden_apps;

                self.filtered_list = apps
                    .into_iter()
                    .filter(|info| {
                        let id = info.id().map(|s| s.to_string()).unwrap_or_default();
                        let is_hidden = is_hidden(&id);
                        if is_hidden && !show_hidden {
                            return false;
                        }
                        filter.is_empty()
                            || info.display_name().to_lowercase().contains(&filter)
                            || info.name().to_lowercase().contains(&filter)
                    })
                    .collect();

                let app_items: Vec<AppItem> = self.filtered_list
                    .iter()
                    .map(|info| {
                        let id = info.id().map(|s| s.to_string()).unwrap_or_default();
                        AppItem {
                            app_info: info.clone(),
                            hidden: is_hidden(&id),
                        }
                    })
                    .collect();

                if !self.filtered_list.is_empty() {
                    self.selected_app = self.filtered_list.first().cloned();
                }

                self.dynamic_box.sender().send(DynamicBoxInput::SetItems(app_items)).unwrap();
                self.update_selected();
            }
            AppLauncherInput::Activate => {
                if let Some(selected_app) = &self.selected_app {
                    launch_detached(selected_app);
                }
                let _ = sender.output(AppLauncherOutput::CloseMenu);
            }
            AppLauncherInput::ParentRevealChanged(revealed) => {
                // If state is changing from hidden to revealed
                if revealed && !self.is_revealed {
                    if let Some(window) = widgets.apps_box.toplevel_window() {
                        window.set_keyboard_mode(KeyboardMode::Exclusive);
                    }
                    sender.input(AppLauncherInput::UpdateAppsList);
                    self.filter = "".to_string();
                    sender.input(AppLauncherInput::FilterChanged("".to_string()));
                    widgets.search_entry.set_text("");
                    widgets.search_entry.grab_focus();
                // if state is change from revealed to hidden
                } else if !revealed && self.is_revealed {
                    if let Some(window) = widgets.apps_box.toplevel_window() {
                        window.set_keyboard_mode(KeyboardMode::None);
                    }
                }
                self.is_revealed = revealed;
            }
            AppLauncherInput::DownPressed => {
                if self.filtered_list.is_empty() {
                    return;
                }
                let current_id = self.selected_app.as_ref().and_then(|s| s.id());
                let current_pos = self.filtered_list.iter().position(|a| a.id() == current_id);
                if let Some(pos) = current_pos {
                    if pos + 1 < self.filtered_list.len() {
                        self.selected_app = self.filtered_list[pos + 1].clone().into();
                    } else {
                        return;
                    }
                } else {
                    return;
                }

                self.update_selected();
                self.ensure_selected_visible(&widgets.scrolled_window);
            }
            AppLauncherInput::UpPressed => {
                if self.filtered_list.is_empty() {
                    return;
                }
                let current_id = self.selected_app.as_ref().and_then(|s| s.id());
                let current_pos = self.filtered_list.iter().position(|a| a.id() == current_id);
                if let Some(pos) = current_pos {
                    if pos > 0 {
                        self.selected_app = self.filtered_list[pos - 1].clone().into();
                    } else {
                        return;
                    }
                } else {
                    return;
                }

                self.update_selected();
                self.ensure_selected_visible(&widgets.scrolled_window);
            }
            AppLauncherInput::HiddenAppsChanged => {
                self.dynamic_box.model().for_each_entry(|_, entry| {
                    if let Some(ctrl) = entry.controller.as_ref().downcast_ref::<Controller<AppLauncherItemModel>>() {
                        let id = ctrl.model().app_info.id().map(|s| s.to_string()).unwrap_or_default();
                        let is_hidden = is_hidden(&id);
                        if ctrl.model().hidden != is_hidden {
                            info!("hidden changed");
                            let _ = ctrl.sender().send(AppLauncherItemInput::HiddenChanged(is_hidden));
                        }
                    }
                });
                sender.input(AppLauncherInput::FilterChanged(self.filter.clone()));
            }
            AppLauncherInput::HideApp(id) => {
                hide_app(id);
            }
            AppLauncherInput::UnhideApp(id) => {
                unhide_app(id);
            }
            AppLauncherInput::CloseMenu => {
                let _ = sender.output(AppLauncherOutput::CloseMenu);
            }
            AppLauncherInput::ShowHiddenAppsChanged => {
                self.show_hidden_apps = !self.show_hidden_apps;
                sender.input(AppLauncherInput::FilterChanged(self.filter.clone()));
            }
            AppLauncherInput::ThemeChanged => {
                let theme = config_manager().config().theme().icons().app_icon_theme().get_untracked();
                let apply_theme = config_manager().config().theme().icons().apply_theme_filter().get_untracked();
                let color_theme = config_manager().config().theme().theme().get_untracked();
                let filter_strength = config_manager().config().theme().icons().filter_strength().get_untracked().get();
                let monochrome_strength = config_manager().config().theme().icons().monochrome_strength().get_untracked().get();
                let contrast_strength = config_manager().config().theme().icons().contrast_strength().get_untracked().get();

                self.dynamic_box.model().for_each_entry(|_, entry| {
                    if let Some(ctrl) = entry
                        .controller
                        .as_ref()
                        .downcast_ref::<Controller<AppLauncherItemModel>>()
                    {
                        let sender = ctrl.sender().clone();
                        let theme = theme.clone();
                        let color_theme = color_theme.clone();

                        let _ = sender.send(AppLauncherItemInput::ThemeChanged(
                            theme,
                            color_theme,
                            apply_theme,
                            filter_strength,
                            monochrome_strength,
                            contrast_strength,
                        ));
                    }
                });
            }
        }

        self.update_view(widgets, sender);
    }
}

impl AppLauncherModel {
    fn update_selected(&self) {
        let selected_id = self.selected_app.as_ref().and_then(|s| s.id());
        self.dynamic_box.model().for_each_entry(|_, entry| {
            if let Some(ctrl) = entry.controller.as_ref().downcast_ref::<Controller<AppLauncherItemModel>>() {
                ctrl.sender().emit(AppLauncherItemInput::NewSelectedId(selected_id.clone()));
            }
        })
    }

    fn ensure_selected_visible(&self, scrolled_window: &ScrolledWindow) {
        let vadj = scrolled_window.vadjustment();
        let selected_key = self.selected_app.as_ref()
            .and_then(|s| s.id())
            .map(|s| s.to_string());

        let container = self.dynamic_box.widget().clone().upcast::<gtk::Widget>();

        for key in self.dynamic_box.model().order.iter() {
            if Some(key) != selected_key.as_ref() {
                continue;
            }
            if let Some(entry) = self.dynamic_box.model().entries.get(key) {
                if !entry.revealer.is_visible() { return; }
                let Some(bounds) = entry.revealer.compute_bounds(&container) else { return };
                let y = bounds.y() as f64;
                let height = bounds.height() as f64;
                let view_start = vadj.value();
                let view_end = view_start + vadj.page_size();

                if y < view_start {
                    vadj.set_value(y);
                } else if y + height > view_end {
                    vadj.set_value((y + height - vadj.page_size()).max(0.0));
                }
                return;
            }
        }
    }
}