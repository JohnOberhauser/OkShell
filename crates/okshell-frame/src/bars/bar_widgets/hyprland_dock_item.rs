use crate::bars::bar::BarType;
use okshell_cache::pinned_apps::{PinnedApp, pin_app, unpin_app};
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, IconsStoreFields, ThemeStoreFields};
use okshell_config::schema::themes::Themes;
use okshell_services::hyprland_service;
use okshell_utils::app_icon::app_icon::set_icon;
use okshell_utils::app_info::find_app_info;
use okshell_utils::launch::launch_detached;
use okshell_utils::strings::truncate_string;
use reactive_graph::traits::GetUntracked;
use relm4::gtk::gio::DesktopAppInfo;
use relm4::gtk::glib::{self, variant::ToVariant};
use relm4::gtk::prelude::{
    ActionMapExt, AppInfoExt, ButtonExt, OrientableExt, PopoverExt, WidgetExt,
};
use relm4::gtk::{Orientation, gio};
use relm4::{Component, ComponentParts, ComponentSender, Sender, WidgetTemplate, gtk};
use tracing::error;
use wayle_hyprland::{Address, Client};

const MAX_MENU_ITEM_LENGTH: usize = 25;

#[derive(Debug, Clone)]
pub(crate) struct HyprlandDockItemModel {
    pub(crate) class: String,
    app_info: Option<DesktopAppInfo>,
    pub(crate) client_count: i16,
    bar_type: BarType,
    orientation: Orientation,
    pub(crate) is_selected: bool,
    last_selected_address: Option<Address>,
    popover: Option<gtk::PopoverMenu>,
    pub(crate) pinned: bool,
}

#[derive(Debug)]
pub(crate) enum HyprlandDockItemInput {
    LeftClicked,
    RightClicked,
    ThemeChanged(String, Themes, bool, f64, f64, f64),
    ClientCountChanged(i16),
    Selected(Address),
    Unselected,
    PinnedChanged(bool),
}

#[derive(Debug)]
pub(crate) enum HyprlandDockItemOutput {}

pub(crate) struct HyprlandDockItemInit {
    pub(crate) class: String,
    pub(crate) client_count: i16,
    pub(crate) bar_type: BarType,
    pub(crate) orientation: Orientation,
    pub(crate) pinned: bool,
}

#[derive(Debug)]
pub(crate) enum HyprlandDockItemCommandOutput {}

#[relm4::widget_template(pub)]
impl WidgetTemplate for IndicatorDot {
    view! {
        gtk::Box {
            set_can_target: false,
            set_can_focus: false,
            set_hexpand: false,
            set_vexpand: false,
        }
    }
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for IndicatorLine {
    view! {
        gtk::Box {
            set_can_target: false,
            set_can_focus: false,
            set_hexpand: false,
            set_vexpand: false,
        }
    }
}

#[relm4::component(pub)]
impl Component for HyprlandDockItemModel {
    type CommandOutput = HyprlandDockItemCommandOutput;
    type Input = HyprlandDockItemInput;
    type Output = HyprlandDockItemOutput;
    type Init = HyprlandDockItemInit;

    view! {
        #[root]
        #[name = "root"]
        gtk::Box {
            add_css_class: "hyprland-dock-item",

            gtk::Overlay {
                add_overlay = &gtk::Box {
                    add_css_class: "bar-dock-indicator-container",
                    set_orientation: model.orientation,
                    set_can_target: false,
                    set_can_focus: false,
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: match model.bar_type {
                        BarType::Top => {
                            gtk::Align::Center
                        }
                        BarType::Bottom => {
                            gtk::Align::Center
                        }
                        BarType::Left => {
                            gtk::Align::Start
                        }
                        BarType::Right => {
                            gtk::Align::End
                        }
                    },
                    set_valign: match model.bar_type {
                        BarType::Top => {
                            gtk::Align::Start
                        }
                        BarType::Bottom => {
                            gtk::Align::End
                        }
                        BarType::Left => {
                            gtk::Align::Center
                        }
                        BarType::Right => {
                            gtk::Align::Center
                        }
                    },

                    #[template]
                    IndicatorDot {
                        #[watch]
                        set_css_classes: if model.is_selected {
                            &["bar-dock-indicator", "selected"]
                        } else {
                            &["bar-dock-indicator"]
                        },
                        #[watch]
                        set_visible: model.client_count > 0 && model.client_count <= 3,
                    },

                    #[template]
                    IndicatorDot {
                        #[watch]
                        set_css_classes: if model.is_selected {
                            &["bar-dock-indicator", "selected"]
                        } else {
                            &["bar-dock-indicator"]
                        },
                        #[watch]
                        set_visible: model.client_count > 1 && model.client_count <= 3,
                    },

                    #[template]
                    IndicatorDot {
                        #[watch]
                        set_css_classes: if model.is_selected {
                            &["bar-dock-indicator", "selected"]
                        } else {
                            &["bar-dock-indicator"]
                        },
                        #[watch]
                        set_visible: model.client_count > 2 && model.client_count <= 3,
                    },

                    #[template]
                    IndicatorLine {
                        #[watch]
                        set_css_classes: if model.orientation == Orientation::Horizontal {
                            if model.is_selected {
                                &["bar-dock-indicator-line-horizontal", "selected"]
                            } else {
                                &["bar-dock-indicator-line-horizontal"]
                            }
                        } else {
                            if model.is_selected {
                                &["bar-dock-indicator-line-vertical", "selected"]
                            } else {
                                &["bar-dock-indicator-line-vertical"]
                            }
                        },
                        #[watch]
                        set_visible: model.client_count > 3,
                    },
                },

                #[name = "button"]
                gtk::Button {
                    #[watch]
                    set_css_classes: if model.is_selected {
                        &["ok-button-primary", "ok-bar-widget"]
                    } else {
                        &["ok-button-surface", "ok-bar-widget"]
                    },
                    set_hexpand: false,
                    set_vexpand: false,
                    connect_clicked[sender] => move |_| {
                        sender.input(HyprlandDockItemInput::LeftClicked);
                    },
                    add_controller = gtk::GestureClick::builder().button(3).build() {
                        connect_released[sender] => move |_, _, _, _| {
                            sender.input(HyprlandDockItemInput::RightClicked);
                        },
                    },

                    #[name="image"]
                    gtk::Image {
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                    },
                },
            },
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let class = params.class;
        let client_count = params.client_count;

        let app_info = find_app_info(class.as_str());

        let base_config = config_manager().config();

        let model = HyprlandDockItemModel {
            class,
            app_info,
            client_count,
            bar_type: params.bar_type,
            orientation: params.orientation,
            is_selected: false,
            last_selected_address: None,
            popover: None,
            pinned: params.pinned,
        };

        let widgets = view_output!();

        let model_clone = model.clone();
        set_icon(
            &model.app_info,
            &Some(model_clone.class),
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

        model.check_selected(&sender);

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
            HyprlandDockItemInput::LeftClicked => {
                let hyprland = hyprland_service();
                let clients = hyprland.clients.get();
                let mut matching: Vec<_> = clients
                    .iter()
                    .filter(|client| client.class.get() == self.class)
                    .collect();
                matching.sort_by_key(|client| client.address.get());

                // Launch the app if it's not already running
                if matching.is_empty() {
                    if let Some(app) = &self.app_info {
                        launch_detached(&app);
                    }
                    return;
                }

                // If the app is already focused, select the next client if there is one
                if self.is_selected {
                    if let Some(last_selected_address) = &self.last_selected_address {
                        let current_idx = matching
                            .iter()
                            .position(|c| c.address.get() == *last_selected_address);
                        if let Some(idx) = current_idx {
                            let next_idx = (idx + 1) % matching.len();
                            let client_to_focus = matching[next_idx];
                            let client_workspace = client_to_focus.workspace.get();
                            let client_address = client_to_focus.address.get();

                            tokio::spawn(async move {
                                let command = format!("workspace {}", client_workspace.id);
                                if let Err(e) = hyprland.dispatch(&command).await {
                                    error!(error = %e, workspace = client_workspace.id, "Failed to switch workspace");
                                }
                                let command = format!("focuswindow address:0x{}", client_address);
                                if let Err(e) = hyprland.dispatch(&command).await {
                                    error!(error = %e, "Failed to focus client");
                                }
                            });
                            return;
                        }
                    }
                }

                // Select the app.  Try to select a window on this workspace, otherwise any window.
                let matching: Vec<_> = matching.into_iter().cloned().collect();
                tokio::spawn(async move {
                    let active_workspace = hyprland.active_workspace().await;
                    let active_workspace_id = active_workspace.as_ref().map(|ws| ws.id.get());

                    let clients_on_workspace: Vec<_> = matching
                        .iter()
                        .filter(|c| Some(c.workspace.get().id) == active_workspace_id)
                        .collect();

                    let client_to_focus = if clients_on_workspace.is_empty() {
                        &matching[0]
                    } else {
                        clients_on_workspace[0]
                    };

                    let client_workspace_id = client_to_focus.workspace.get().id;
                    let client_address = client_to_focus.address.get();

                    if Some(client_workspace_id) != active_workspace_id {
                        let command = format!("workspace {}", client_workspace_id);
                        if let Err(e) = hyprland.dispatch(&command).await {
                            error!(error = %e, "Failed to switch workspace");
                        }
                    }
                    let command = format!("focuswindow address:0x{}", client_address);
                    if let Err(e) = hyprland.dispatch(&command).await {
                        error!(error = %e, "Failed to focus client");
                    }
                });
            }
            HyprlandDockItemInput::RightClicked => {
                let hyprland = hyprland_service();
                let clients = hyprland.clients.get();
                let matching: Vec<_> = clients
                    .iter()
                    .filter(|client| client.class.get() == self.class)
                    .collect();

                let action_group = gio::SimpleActionGroup::new();
                let menu = gio::Menu::new();

                let menu_title: Option<String> = self
                    .app_info
                    .as_ref()
                    .map(|app| truncate_string(&app.name(), MAX_MENU_ITEM_LENGTH));

                if matching.is_empty() {
                    let mut general_section_title = None;

                    if let Some(app) = &self.app_info {
                        let actions = app.list_actions();
                        if actions.is_empty() {
                            general_section_title = menu_title.clone();
                        } else {
                            let app_actions_section = gio::Menu::new();
                            add_app_actions_to_menu(&app_actions_section, &action_group, app);
                            menu.append_section(menu_title.as_deref(), &app_actions_section);
                        }
                    }

                    let general_section = gio::Menu::new();
                    if let Some(app) = &self.app_info {
                        add_launch_to_menu(&general_section, &action_group, app);
                        if self.pinned {
                            self.add_unpin_to_menu(&general_section, &action_group, app);
                        }
                    }
                    menu.append_section(general_section_title.as_deref(), &general_section);
                } else {
                    let details_section = gio::Menu::new();
                    add_window_details_to_menu(&details_section, &action_group, &self.class);
                    menu.append_section(menu_title.as_deref(), &details_section);

                    if let Some(last_selected_address) = &self.last_selected_address {
                        let focused_client = matching
                            .iter()
                            .find(|c| c.address.get() == *last_selected_address);
                        if let Some(focused) = focused_client {
                            let focused_section = gio::Menu::new();
                            add_move_focused_client_to_menu(
                                &focused_section,
                                &action_group,
                                focused,
                            );
                            add_close_focused_to_menu(&focused_section, &action_group, focused);
                            menu.append_section(None, &focused_section);
                        }
                    }

                    if let Some(app) = &self.app_info {
                        let app_actions_section = gio::Menu::new();
                        add_app_actions_to_menu(&app_actions_section, &action_group, app);
                        menu.append_section(None, &app_actions_section);
                    }

                    let general_section = gio::Menu::new();
                    if let Some(app) = &self.app_info {
                        add_launch_to_menu(&general_section, &action_group, app);

                        if self.pinned {
                            self.add_unpin_to_menu(&general_section, &action_group, app);
                        } else {
                            self.add_pin_to_menu(&general_section, &action_group, app);
                        }
                    }

                    add_quit_to_menu(&general_section, &action_group, &self.class);
                    menu.append_section(None, &general_section);
                }

                let popover = gtk::PopoverMenu::from_model(Some(&menu));
                popover.set_has_arrow(false);
                popover.insert_action_group("main", Some(&action_group));
                popover.set_parent(&widgets.root);

                popover.popup();

                self.popover = Some(popover);
            }
            HyprlandDockItemInput::ThemeChanged(
                theme,
                color_theme,
                apply_theme,
                filter_strength,
                monochrome_strength,
                contrast_strength,
            ) => {
                let class = self.class.clone();
                set_icon(
                    &self.app_info,
                    &Some(class),
                    &widgets.image,
                    theme,
                    &color_theme,
                    apply_theme,
                    filter_strength,
                    monochrome_strength,
                    contrast_strength,
                );
            }
            HyprlandDockItemInput::ClientCountChanged(count) => {
                self.client_count = count;
            }
            HyprlandDockItemInput::Selected(address) => {
                self.is_selected = true;
                self.last_selected_address = Some(address);
            }
            HyprlandDockItemInput::Unselected => {
                self.is_selected = false;
                self.last_selected_address = None;
            }
            HyprlandDockItemInput::PinnedChanged(pinned) => {
                self.pinned = pinned;
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

impl HyprlandDockItemModel {
    fn check_selected(&self, sender: &ComponentSender<Self>) {
        let class = self.class.clone();
        let sender = sender.clone();
        tokio::spawn(async move {
            let hyprland = hyprland_service();
            let active_client = hyprland.active_window().await;
            if let Some(active_client) = active_client {
                let clients = hyprland.clients.get();
                let is_selected = clients
                    .iter()
                    .filter(|client| client.class.get() == class)
                    .any(|client| *client == active_client);
                if is_selected {
                    sender.input(HyprlandDockItemInput::Selected(active_client.address.get()));
                }
            }
        });
    }

    fn add_pin_to_menu(
        &self,
        menu: &gio::Menu,
        action_group: &gio::SimpleActionGroup,
        app: &DesktopAppInfo,
    ) {
        let action = gio::SimpleAction::new("pin", None);
        let app = app.clone();
        let class = self.class.clone();
        action.connect_activate(move |_, _| {
            pin_app(PinnedApp {
                desktop_id: app.id().map(|id| id.to_string()).unwrap_or_default(),
                hyprland_class: class.clone(),
            });
        });
        action_group.add_action(&action);
        menu.append(Some("Pin to dock"), Some("main.pin"));
    }

    fn add_unpin_to_menu(
        &self,
        menu: &gio::Menu,
        action_group: &gio::SimpleActionGroup,
        app: &DesktopAppInfo,
    ) {
        let action = gio::SimpleAction::new("unpin", None);
        let app = app.clone();
        action.connect_activate(move |_, _| {
            unpin_app(
                app.id()
                    .map(|id| id.to_string())
                    .unwrap_or_default()
                    .as_str(),
            );
        });
        action_group.add_action(&action);
        menu.append(Some("Unpin"), Some("main.unpin"));
    }
}

fn add_launch_to_menu(
    menu: &gio::Menu,
    action_group: &gio::SimpleActionGroup,
    app: &DesktopAppInfo,
) {
    let action = gio::SimpleAction::new("launch", None);
    let app = app.clone();
    action.connect_activate(move |_, _| {
        launch_detached(&app);
    });
    action_group.add_action(&action);
    menu.append(Some("Launch"), Some("main.launch"));
}

fn add_app_actions_to_menu(
    menu: &gio::Menu,
    action_group: &gio::SimpleActionGroup,
    app: &DesktopAppInfo,
) {
    let actions = app.list_actions();
    for (index, action_id) in actions.iter().enumerate() {
        let action_name = format!("action{}", index);
        let action = gio::SimpleAction::new(&action_name, None);
        let label = app.action_name(&action_id).to_string();

        let app = app.clone();
        let action_id = action_id.clone();
        action.connect_activate(move |_, _| {
            app.launch_action(&action_id, None::<&gio::AppLaunchContext>);
        });
        action_group.add_action(&action);
        menu.append(Some(&label), Some(&format!("main.{}", action_name)));
    }
}

fn add_move_focused_client_to_menu(
    menu: &gio::Menu,
    action_group: &gio::SimpleActionGroup,
    focused_client: &Client,
) {
    let hyprland = hyprland_service();

    let workspaces = hyprland.workspaces.get();
    if workspaces.len() <= 1 {
        return;
    }

    let action = gio::SimpleAction::new("move-focused", Some(glib::VariantTy::INT32));
    let hyprland_clone = hyprland.clone();
    action.connect_activate(move |_, param| {
        if let Some(param) = param {
            let target_workspace_id = param.get::<i32>().unwrap() as i64;
            let workspaces = hyprland_clone.workspaces.get();
            if let Some(workspace) = workspaces
                .iter()
                .find(|ws| ws.id.get() == target_workspace_id)
            {
                let workspace_id = workspace.id.get();
                let hyprland_clone = hyprland.clone();
                tokio::spawn(async move {
                    let command = format!("movetoworkspace {}", workspace_id);
                    if let Err(e) = hyprland_clone.dispatch(&command).await {
                        error!(error = %e, "Failed to switch workspace");
                    }
                });
            }
        }
    });
    action_group.add_action(&action);

    let current_ws_id = focused_client.workspace.get().id;
    let mut workspace_ids: Vec<_> = workspaces.iter().map(|ws| ws.id.get()).collect();
    workspace_ids.sort();

    let submenu = gio::Menu::new();
    for id in workspace_ids {
        if id == current_ws_id {
            continue;
        }
        let item = gio::MenuItem::new(Some(&format!("Workspace {}", id)), None);
        item.set_action_and_target_value(
            Some("main.move-focused"),
            Some(&(id as i32).to_variant()),
        );
        submenu.append_item(&item);
    }

    let menu_item = gio::MenuItem::new(Some("Move focused"), None);
    menu_item.set_submenu(Some(&submenu));
    menu.append_item(&menu_item);
}

fn add_close_focused_to_menu(
    menu: &gio::Menu,
    action_group: &gio::SimpleActionGroup,
    focused_client: &Client,
) {
    let action = gio::SimpleAction::new("close-focused", None);
    let focused_client_address = focused_client.address.get();
    action.connect_activate(move |_, _| {
        let address = focused_client_address.clone();
        tokio::spawn(async move {
            let hyprland = hyprland_service();
            let command = format!("closewindow address:0x{}", address);
            if let Err(e) = hyprland.dispatch(&command).await {
                error!(error = %e, "Failed to switch workspace");
            }
        });
    });
    action_group.add_action(&action);
    menu.append(Some("Close Focused"), Some("main.close-focused"));
}

fn add_quit_to_menu(menu: &gio::Menu, action_group: &gio::SimpleActionGroup, class: &str) {
    let action = gio::SimpleAction::new("quit", None);
    let class = class.to_string();
    action.connect_activate(move |_, _| {
        let hyprland = hyprland_service();
        let clients = hyprland.clients.get();
        let matching: Vec<_> = clients
            .iter()
            .filter(|client| client.class.get() == class)
            .collect();
        for client in matching {
            let address = client.address.get().clone();
            tokio::spawn(async move {
                let hyprland = hyprland_service();
                if let Err(e) = hyprland
                    .dispatch(&format!("closewindow address:0x{}", address))
                    .await
                {
                    error!(error = %e, "Failed to close window");
                }
            });
        }
    });
    action_group.add_action(&action);
    menu.append(Some("Quit"), Some("main.quit"));
}

fn add_window_details_to_menu(
    menu: &gio::Menu,
    action_group: &gio::SimpleActionGroup,
    class: &str,
) {
    let hyprland = hyprland_service();
    let clients = hyprland.clients.get();
    let matching: Vec<_> = clients
        .iter()
        .filter(|client| client.class.get() == class)
        .collect();

    for (index, client) in matching.iter().enumerate() {
        let action_name = format!("details{}", index);
        let action = gio::SimpleAction::new(&action_name, None);
        let address = client.address.get().clone();
        action.connect_activate(move |_, _| {
            let address = address.clone();
            tokio::spawn(async move {
                let hyprland = hyprland_service();
                if let Err(e) = hyprland
                    .dispatch(&format!("focuswindow address:0x{}", address))
                    .await
                {
                    error!(error = %e, "Failed to focus client");
                }
            });
        });
        action_group.add_action(&action);
        let title = truncate_string(&client.title.get(), MAX_MENU_ITEM_LENGTH);
        menu.append(Some(&title), Some(&format!("main.{}", action_name)));
    }
}
