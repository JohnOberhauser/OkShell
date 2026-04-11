use std::sync::Arc;
use futures::StreamExt;
use reactive_graph::traits::*;
use relm4::{
    gtk,
    gtk::prelude::*,
    Component,
    ComponentController,
    ComponentParts,
    ComponentSender,
    Controller
};
use relm4::gtk::{Orientation, RevealerTransitionType};
use okshell_common::watch;
use wayle_hyprland::{Address, Client, HyprlandEvent};
use okshell_cache::pinned_apps::{pinned_apps_store, PinnedAppsStateStoreFields};
use okshell_common::scoped_effects::EffectScope;
use okshell_services::hyprland_service;
use crate::bars::bar::BarType;
use crate::bars::bar_widgets::app_launcher::{AppLauncherInit, AppLauncherModel, AppLauncherOutput};
use crate::bars::bar_widgets::hyprland_dock::HyprlandDockOutput::AppLauncherClicked;
use crate::bars::bar_widgets::hyprland_dock_item::{HyprlandDockItemInit, HyprlandDockItemInput, HyprlandDockItemModel};
use okshell_common::dynamic_box::dynamic_box::{DynamicBoxFactory, DynamicBoxInit, DynamicBoxInput, DynamicBoxModel, DynamicBoxOutput};
use okshell_common::dynamic_box::generic_widget_controller::GenericWidgetController;
use okshell_common::dynamic_box::generic_widget_controller::GenericWidgetControllerExtSafe;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, ThemeStoreFields};

#[derive(Clone, Debug)]
pub struct DockItem {
    class: String,
    client_count: i16,
    pinned: bool,
}

pub(crate) struct HyprlandDockModel {
    dynamic_box: Controller<DynamicBoxModel<DockItem, String>>,
    orientation: Orientation,
    app_launcher_controller: Controller<AppLauncherModel>,
    _effect: EffectScope,
}

#[derive(Debug)]
pub(crate) enum HyprlandDockInput {
    ThemeChanged(String),
    OnReordered(Vec<String>),
}

#[derive(Debug)]
pub(crate) enum HyprlandDockOutput {
    AppLauncherClicked,
}

pub(crate) struct HyprlandDockInit {
    pub(crate) orientation: Orientation,
    pub(crate) bar_type: BarType,
}

#[derive(Debug)]
pub(crate) enum HyprlandDockCommandOutput {
    ClientsChanged(Vec<Arc<Client>>),
    ActiveWindowChanged(Address),
}

#[relm4::component(pub)]
impl Component for HyprlandDockModel {
    type CommandOutput = HyprlandDockCommandOutput;
    type Input = HyprlandDockInput;
    type Output = HyprlandDockOutput;
    type Init = HyprlandDockInit;

    view! {
        #[root]
        #[name = "root"]
        gtk::Box {
            add_css_class: "hyprland-dock-bar-widget",
            set_orientation: model.orientation,
            set_hexpand: model.orientation == Orientation::Vertical,
            set_vexpand: model.orientation == Orientation::Horizontal,

            model.dynamic_box.widget().clone() {},

            model.app_launcher_controller.widget().clone() {},
        },
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        Self::spawn_main_watcher(&sender);
        Self::spawn_active_window_watcher(&sender);

        let factory = DynamicBoxFactory::<DockItem, String> {
            id: Box::new(|item| item.class.clone()),
            create: Box::new(move |item| {
                let controller: Controller<HyprlandDockItemModel> =
                    HyprlandDockItemModel::builder()
                        .launch(HyprlandDockItemInit {
                            class: item.class.clone(),
                            client_count: item.client_count,
                            bar_type: params.bar_type,
                            orientation: params.orientation,
                            pinned: item.pinned,
                        })
                        .detach();
                Box::new(controller) as Box<dyn GenericWidgetController>
            }),
            update: None,
        };

        let transition_type = if params.orientation == Orientation::Horizontal {
            RevealerTransitionType::SwingLeft
        } else {
            RevealerTransitionType::SwingUp
        };

        let dynamic: Controller<DynamicBoxModel<DockItem, String>> =
            DynamicBoxModel::builder()
                .launch(DynamicBoxInit{
                    factory,
                    orientation: params.orientation,
                    spacing: 0,
                    transition_type,
                    transition_duration_ms: 200,
                    reverse: false,
                    retain_entries: false,
                    allow_drag_and_drop: true,
                })
                .forward(sender.input_sender(), |msg| {
                    match msg { DynamicBoxOutput::Reordered(keys) => {
                        HyprlandDockInput::OnReordered(keys)
                    } }
                });

        let app_launcher_controller = AppLauncherModel::builder()
            .launch(AppLauncherInit{
                orientation: params.orientation,
            })
            .forward(sender.output_sender(), |msg| {
                match msg {
                    AppLauncherOutput::Clicked => {
                        AppLauncherClicked
                    }
                }
            });

        let mut effects = EffectScope::new();

        let pinned_apps_store = pinned_apps_store();
        let sender_clone = sender.clone();
        effects.push(move |_| {
            let store = pinned_apps_store.clone();
            let _ = store.apps().get();
            let hyprland = hyprland_service();
            let clients = hyprland.clients.get();
            let _ = sender_clone.command_sender().send(HyprlandDockCommandOutput::ClientsChanged(clients));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let icon_theme = config_manager().config().theme().app_icon_theme().get();
            sender_clone.input(HyprlandDockInput::ThemeChanged(icon_theme));
        });

        let model = HyprlandDockModel {
            dynamic_box: dynamic,
            orientation: params.orientation,
            app_launcher_controller,
            _effect: effects,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            HyprlandDockInput::ThemeChanged(theme) => {
                self.dynamic_box.model().for_each_entry(|_, entry| {
                    if let Some(ctrl) = entry
                        .controller
                        .as_ref()
                        .downcast_ref::<Controller<HyprlandDockItemModel>>()
                    {
                        let theme = theme.clone();
                        let _ = ctrl
                            .sender()
                            .send(HyprlandDockItemInput::ThemeChanged(theme));
                    }
                });
            }
            HyprlandDockInput::OnReordered(classes_in_new_order) => {
                let store = pinned_apps_store();
                let current_pinned = store.read_untracked().apps.clone();

                let pinned_map: std::collections::HashMap<&str, _> = current_pinned
                    .iter()
                    .map(|app| (app.hyprland_class.as_str(), app))
                    .collect();

                let reordered_pinned: Vec<_> = classes_in_new_order
                    .iter()
                    .filter_map(|class| pinned_map.get(class.as_str()).copied().cloned())
                    .collect();

                if reordered_pinned.iter().map(|a| &a.hyprland_class).collect::<Vec<_>>()
                    != current_pinned.iter().map(|a| &a.hyprland_class).collect::<Vec<_>>()
                {
                    store.write().apps = reordered_pinned;
                }
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            HyprlandDockCommandOutput::ClientsChanged(clients) => {
                let pinned_apps = pinned_apps_store().read_untracked().apps.clone();

                let mut sorted_clients = clients.to_vec();
                sorted_clients.sort_by_key(|c| c.pid.get());

                let mut counts: std::collections::HashMap<String, i16> = std::collections::HashMap::new();
                for client in &sorted_clients {
                    *counts.entry(client.class.get().to_string()).or_insert(0) += 1;
                }

                // Build pinned rows first (may have 0 running clients).
                let mut seen = std::collections::HashSet::new();
                let mut rows: Vec<DockItem> = pinned_apps
                    .iter()
                    .map(|app| {
                        seen.insert(app.hyprland_class.clone());
                        DockItem {
                            class: app.hyprland_class.clone(),
                            client_count: *counts.get(&app.hyprland_class).unwrap_or(&0),
                            pinned: true,
                        }
                    })
                    .collect();

                // Then append running apps that aren't pinned.
                let unpinned_rows = sorted_clients
                    .iter()
                    .map(|client| client.class.get().to_string())
                    .filter(|class| seen.insert(class.clone()))
                    .map(|class| DockItem {
                        client_count: *counts.get(&class).unwrap_or(&0),
                        class,
                        pinned: false,
                    });
                rows.extend(unpinned_rows);

                self.dynamic_box.sender().send(DynamicBoxInput::SetItems(rows)).unwrap();

                // Update each entry's client count and pinned state.
                let pinned_classes: std::collections::HashSet<&str> = pinned_apps
                    .iter()
                    .map(|a| a.hyprland_class.as_str())
                    .collect();

                self.dynamic_box.model().for_each_entry(|_, entry| {
                    if let Some(ctrl) = entry
                        .controller
                        .as_ref()
                        .downcast_ref::<Controller<HyprlandDockItemModel>>()
                    {
                        let model = ctrl.model();
                        let count = *counts.get(&model.class).unwrap_or(&0);
                        if model.client_count != count {
                            let _ = ctrl
                                .sender()
                                .send(HyprlandDockItemInput::ClientCountChanged(count));
                        }
                        let is_pinned = pinned_classes.contains(model.class.as_str());
                        if model.pinned != is_pinned {
                            let _ = ctrl
                                .sender()
                                .send(HyprlandDockItemInput::PinnedChanged(is_pinned));
                        }
                    }
                });
            }
            HyprlandDockCommandOutput::ActiveWindowChanged(address) => {
                let hyprland = hyprland_service();
                let clients = hyprland.clients.get();
                self.dynamic_box.model().for_each_entry(|_, entry| {
                    if let Some(ctrl) = entry
                        .controller
                        .as_ref()
                        .downcast_ref::<Controller<HyprlandDockItemModel>>()
                    {
                        let model = ctrl.model();
                        let is_selected = clients
                            .iter()
                            .filter(|client| client.class.get() == model.class)
                            .any(|client| client.address.get() == address);
                        if is_selected {
                            let _ = ctrl
                                .sender()
                                .send(HyprlandDockItemInput::Selected(address.clone()));
                        } else {
                            let _ = ctrl
                                .sender()
                                .send(HyprlandDockItemInput::Unselected);
                        }
                    }
                });
            }
        }
    }
}

impl HyprlandDockModel {
    fn spawn_main_watcher(
        sender: &ComponentSender<Self>,
    ) {
        let hyprland = hyprland_service();
        let clients = hyprland.clients.clone();

        watch!(sender, [clients.watch()], |out| {
            let _ = out.send(HyprlandDockCommandOutput::ClientsChanged(clients.get()));
        })
    }

    fn spawn_active_window_watcher(
        sender: &ComponentSender<Self>,
    ) {
        sender.command(move |out, shutdown| {
            async move {
                let hyprland = hyprland_service();
                let mut events = hyprland.events();
                let shutdown_fut = shutdown.wait();
                tokio::pin!(shutdown_fut);

                loop {
                    tokio::select! {
                        () = &mut shutdown_fut => return,
                        event = events.next() => {
                            let Some(event) = event else { continue; };
                            match event {
                                HyprlandEvent::ActiveWindowV2 { address } => {
                                    let _ = out.send(HyprlandDockCommandOutput::ActiveWindowChanged(address));
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        });
    }
}