use okshell_services::hyprland_service;
use relm4::gtk::gio::prelude::ActionMapExt;
use relm4::gtk::prelude::{ButtonExt, PopoverExt, WidgetExt};
use relm4::gtk::{Orientation, gio};
use relm4::{Component, ComponentParts, ComponentSender, gtk};
use tracing::error;

#[derive(Debug)]
pub(crate) struct HyprlandLayoutModel {
    orientation: Orientation,
    popover: Option<gtk::PopoverMenu>,
}

#[derive(Debug)]
pub(crate) enum HyprlandLayoutInput {
    Clicked,
    SetLayout(&'static str),
}

#[derive(Debug)]
pub(crate) enum HyprlandLayoutOutput {}

pub(crate) struct HyprlandLayoutInit {
    pub(crate) orientation: Orientation,
}

#[derive(Debug)]
pub(crate) enum HyprlandLayoutCommandOutput {}

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

            #[name = "button"]
            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-bar-widget"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(HyprlandLayoutInput::Clicked);
                },

                #[name="image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("layout-symbolic"),
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HyprlandLayoutModel {
            orientation: params.orientation,
            popover: None,
        };

        let widgets = view_output!();

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
            HyprlandLayoutInput::Clicked => {
                let action_group = gio::SimpleActionGroup::new();
                let menu = gio::Menu::new();

                Self::add_layout(&sender, &menu, &action_group, "dwindle", "Dwindle");
                Self::add_layout(&sender, &menu, &action_group, "master", "Master");
                Self::add_layout(&sender, &menu, &action_group, "scrolling", "Scrolling");
                Self::add_layout(&sender, &menu, &action_group, "monocle", "Monocle");

                let popover = gtk::PopoverMenu::from_model(Some(&menu));
                popover.set_has_arrow(false);
                popover.insert_action_group("main", Some(&action_group));
                popover.set_parent(&widgets.button);

                popover.popup();

                self.popover = Some(popover);

                widgets.button.unset_state_flags(gtk::StateFlags::PRELIGHT);
            }
            HyprlandLayoutInput::SetLayout(layout) => {
                tokio::spawn(async move {
                    let hyprland = hyprland_service();
                    if let Some(active_workspace) = hyprland.active_workspace().await {
                        let workspace_id = active_workspace.id.get();
                        let command = format!("workspace {}, layout:{}", workspace_id, layout);
                        let result = hyprland.keyword(&command).await;
                        if let Err(e) = result {
                            error!(error = %e, workspace = workspace_id, "Failed set workspace layout");
                        }
                    }
                });
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
    ) {
        let action = gio::SimpleAction::new(id, None);
        let sender = sender.clone();
        action.connect_activate(move |_, _| {
            let _ = sender.input(HyprlandLayoutInput::SetLayout(id));
        });
        action_group.add_action(&action);
        menu.append(Some(name), Some(format!("main.{}", id).as_str()));
    }
}
