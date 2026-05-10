use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{
    BarWidgetsStoreFields, BarsStoreFields, ConfigStoreFields, QuickSettingsBarWidgetStoreFields,
};
use okshell_config::schema::quick_settings_icon::QuickSettingsIcon;
use reactive_graph::traits::{Get, GetUntracked};
use relm4::gtk::Orientation;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use relm4::{Component, ComponentParts, ComponentSender, gtk};

#[derive(Debug, Clone)]
pub(crate) struct QuickSettingsModel {
    orientation: Orientation,
    icon_name: String,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum QuickSettingsInput {
    IconEffect(QuickSettingsIcon),
}

#[derive(Debug)]
pub(crate) enum QuickSettingOutput {
    Clicked,
}

pub(crate) struct QuickSettingsInit {
    pub(crate) orientation: Orientation,
}

#[relm4::component(pub)]
impl Component for QuickSettingsModel {
    type CommandOutput = ();
    type Input = QuickSettingsInput;
    type Output = QuickSettingOutput;
    type Init = QuickSettingsInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "quick-settings-bar-widget",
            set_hexpand: model.orientation == Orientation::Vertical,
            set_vexpand: model.orientation == Orientation::Horizontal,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,

            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-bar-widget"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.output(QuickSettingOutput::Clicked).unwrap_or_default();
                },

                #[name="image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    #[watch]
                    set_icon_name: Some(model.icon_name.as_str()),
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut effects = EffectScope::new();

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let value = config_manager()
                .config()
                .bars()
                .widgets()
                .quick_settings()
                .icon()
                .get();
            sender_clone.input(QuickSettingsInput::IconEffect(value));
        });

        let model = QuickSettingsModel {
            orientation: params.orientation,
            icon_name: get_icon_name(
                config_manager()
                    .config()
                    .bars()
                    .widgets()
                    .quick_settings()
                    .icon()
                    .get_untracked(),
            )
            .to_string(),
            _effects: effects,
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
            QuickSettingsInput::IconEffect(icon) => {
                self.icon_name = get_icon_name(icon).to_string();
            }
        }

        self.update_view(widgets, sender);
    }
}

fn get_icon_name(icon: QuickSettingsIcon) -> &'static str {
    match icon {
        QuickSettingsIcon::Arch => "arch-symbolic",
        QuickSettingsIcon::Fedora => "fedora-symbolic",
        QuickSettingsIcon::Hyprland => "hyprland-symbolic",
        QuickSettingsIcon::Nix => "nix-symbolic",
    }
}
