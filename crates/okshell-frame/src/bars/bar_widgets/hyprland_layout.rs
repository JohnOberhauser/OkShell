use futures::StreamExt;
use okshell_services::hyprland_service;
use okshell_utils::hyprland::get_active_workspaces;
use relm4::gtk::gdk::prelude::{DisplayExt, MonitorExt};
use relm4::gtk::gio::prelude::ActionMapExt;
use relm4::gtk::glib::clone::Downgrade;
use relm4::gtk::glib::object::IsA;
use relm4::gtk::glib::variant::ToVariant;
use relm4::gtk::prelude::{BoxExt, ButtonExt, NativeExt, PopoverExt, WidgetExt};
use relm4::gtk::{Orientation, gio};
use relm4::{Component, ComponentParts, ComponentSender, gtk};
use tracing::error;
use wayle_hyprland::HyprlandEvent;

#[derive(Debug)]
pub(crate) struct HyprlandLayoutModel {
    orientation: Orientation,
    icon: String,
}

#[derive(Debug)]
pub(crate) enum HyprlandLayoutInput {
    SetLayout(&'static str),
    RefreshIcon,
    SetIcon(String),
}

#[derive(Debug)]
pub(crate) enum HyprlandLayoutOutput {}

pub(crate) struct HyprlandLayoutInit {
    pub(crate) orientation: Orientation,
}

#[derive(Debug)]
pub(crate) enum HyprlandLayoutCommandOutput {
    UpdateIcon,
}

#[relm4::component(pub)]
impl Component for HyprlandLayoutModel {
    type CommandOutput = HyprlandLayoutCommandOutput;
    type Input = HyprlandLayoutInput;
    type Output = HyprlandLayoutOutput;
    type Init = HyprlandLayoutInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "hyprland-layout-bar-widget",
            set_hexpand: model.orientation == Orientation::Vertical,
            set_vexpand: model.orientation == Orientation::Horizontal,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,

            #[name = "menu_button"]
            gtk::MenuButton {
                set_css_classes: &["ok-button-surface", "ok-bar-widget"],
                set_hexpand: false,
                set_vexpand: false,
                #[watch]
                set_icon_name: model.icon.as_str(),
                set_always_show_arrow: false,
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        Self::spawn_main_watcher(&sender);

        let model = HyprlandLayoutModel {
            orientation: params.orientation,
            icon: "layout-symbolic".to_string(),
        };

        let widgets = view_output!();

        let action_group = gio::SimpleActionGroup::new();
        let menu = gio::Menu::new();

        let layouts = [
            Self::add_layout(
                &sender,
                &menu,
                &action_group,
                "dwindle",
                "Dwindle",
                "layout-dwindle-symbolic",
            ),
            Self::add_layout(
                &sender,
                &menu,
                &action_group,
                "master",
                "Master",
                "layout-master-symbolic",
            ),
            Self::add_layout(
                &sender,
                &menu,
                &action_group,
                "scrolling",
                "Scrolling",
                "layout-scrolling-symbolic",
            ),
            Self::add_layout(
                &sender,
                &menu,
                &action_group,
                "monocle",
                "Monocle",
                "layout-monocle-symbolic",
            ),
        ];

        let popover = gtk::PopoverMenu::from_model_full(&menu, gtk::PopoverMenuFlags::NESTED);
        popover.set_has_arrow(false);

        for (custom_id, widget) in &layouts {
            popover.add_child(widget, custom_id);
        }

        widgets.menu_button.set_popover(Some(&popover));
        widgets
            .menu_button
            .insert_action_group("main", Some(&action_group));

        for (custom_id, button) in &layouts {
            popover.add_child(button, custom_id);
            let popover_weak = popover.downgrade();
            button.connect_clicked(move |_| {
                if let Some(p) = popover_weak.upgrade() {
                    p.popdown();
                }
            });
        }

        sender.input(HyprlandLayoutInput::RefreshIcon);

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            HyprlandLayoutInput::SetLayout(layout) => {
                let sender_clone = sender.clone();
                tokio::spawn(async move {
                    let hyprland = hyprland_service();
                    if let Some(active_workspace) = hyprland.active_workspace().await {
                        let workspace_id = active_workspace.id.get();
                        let command = format!(
                            "hl.workspace_rule({{ workspace = \"{}\", layout = \"{}\"}})",
                            workspace_id, layout
                        );
                        let result = hyprland.eval(&command).await;
                        if let Err(e) = result {
                            error!(error = %e, workspace = workspace_id, "Failed set workspace layout");
                        }
                    }
                    sender_clone.input(HyprlandLayoutInput::SetIcon(layout.to_string()));
                });
            }
            HyprlandLayoutInput::RefreshIcon => {
                let hyprland = hyprland_service();
                let active_workspaces = get_active_workspaces();
                let workspaces = hyprland.workspaces.get();
                let Some(active_connector) = Self::connector_for_widget(root) else {
                    return;
                };

                let workspace = workspaces
                    .iter()
                    .filter(|w| w.monitor.get() == active_connector)
                    .find(|w| active_workspaces.iter().any(|aw| aw.id == w.id.get()));

                if let Some(workspace) = workspace {
                    let tiled_layout_name = workspace.tiled_layout.get();
                    sender.input(HyprlandLayoutInput::SetIcon(tiled_layout_name));
                }
            }
            HyprlandLayoutInput::SetIcon(layout) => match layout.as_str() {
                "dwindle" => {
                    self.icon = "layout-dwindle-symbolic".to_string();
                }
                "master" => {
                    self.icon = "layout-master-symbolic".to_string();
                }
                "scrolling" => {
                    self.icon = "layout-scrolling-symbolic".to_string();
                }
                "monocle" => {
                    self.icon = "layout-monocle-symbolic".to_string();
                }
                _ => {
                    self.icon = "layout-symbolic".to_string();
                }
            },
        }

        self.update_view(widgets, sender);
    }

    fn update_cmd_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            HyprlandLayoutCommandOutput::UpdateIcon => {
                sender.input(HyprlandLayoutInput::RefreshIcon);
            }
        }
    }
}

impl HyprlandLayoutModel {
    fn add_layout(
        sender: &ComponentSender<Self>,
        menu: &gio::Menu,
        action_group: &gio::SimpleActionGroup,
        id: &'static str,
        name: &'static str,
        icon_name: &str,
    ) -> (String, gtk::Button) {
        let action = gio::SimpleAction::new(id, None);
        let sender_clone = sender.clone();
        action.connect_activate(move |_, _| {
            let _ = sender_clone.input(HyprlandLayoutInput::SetLayout(id));
        });
        action_group.add_action(&action);

        let custom_id = format!("layout-{}", id);

        let item = gio::MenuItem::new(Some(name), Some(&format!("main.{}", id)));
        item.set_attribute_value("custom", Some(&custom_id.to_variant()));
        menu.append_item(&item);

        let row = gtk::Box::builder()
            .orientation(Orientation::Horizontal)
            .spacing(8)
            .build();
        row.append(&gtk::Image::from_icon_name(icon_name));
        row.append(&gtk::Label::new(Some(name)));

        let button = gtk::Button::builder()
            .child(&row)
            .action_name(&format!("main.{}", id))
            .css_classes(["ok-button-surface"])
            .build();

        (custom_id, button)
    }

    fn spawn_main_watcher(sender: &ComponentSender<Self>) {
        sender.command(move |out, shutdown| async move {
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
                            HyprlandEvent::WorkspaceV2 { .. } => {
                                let _ = out.send(HyprlandLayoutCommandOutput::UpdateIcon);
                            }
                            _ => {}
                        }
                    }
                }
            }
        });
    }

    fn connector_for_widget(widget: &impl IsA<gtk::Widget>) -> Option<String> {
        let surface = widget.root()?.surface()?;
        widget
            .display()
            .monitor_at_surface(&surface)?
            .connector()
            .map(|s| s.to_string())
    }
}
