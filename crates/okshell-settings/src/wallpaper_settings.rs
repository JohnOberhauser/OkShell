use reactive_graph::prelude::Get;
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::gio;
use relm4::gtk::prelude::{BoxExt, ButtonExt, FileExt, OrientableExt, WidgetExt};
use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, WallpaperStoreFields};

#[derive(Debug, Clone)]
pub(crate) struct WallpaperSettingsModel {
    wallpaper_directory: String,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum WallpaperSettingsInput {
    ChangeWallpaperDirectoryClicked,
    WallpaperDirectoryChanged(String),
}

#[derive(Debug)]
pub(crate) enum WallpaperSettingsOutput {}

pub(crate) struct WallpaperSettingsInit {}

#[derive(Debug)]
pub(crate) enum WallpaperSettingsCommandOutput {}

#[relm4::component(pub)]
impl Component for WallpaperSettingsModel {
    type CommandOutput = WallpaperSettingsCommandOutput;
    type Input = WallpaperSettingsInput;
    type Output = WallpaperSettingsOutput;
    type Init = WallpaperSettingsInit;

    view! {
        #[root]
        gtk::ScrolledWindow {
            set_vscrollbar_policy: gtk::PolicyType::Automatic,
            set_hscrollbar_policy: gtk::PolicyType::Never,
            set_propagate_natural_height: false,
            set_propagate_natural_width: false,
            set_hexpand: true,
            set_vexpand: true,
            gtk::Box {
                add_css_class: "settings-page",
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_spacing: 16,

                gtk::Label {
                    add_css_class: "label-large-bold",
                    set_label: "Wallpaper Directory",
                    set_halign: gtk::Align::Start,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Label {
                        add_css_class: "label-small",
                        #[watch]
                        set_label: model.wallpaper_directory.as_str(),
                        set_halign: gtk::Align::Start,
                        set_hexpand: true,
                        set_xalign: 0.0,
                        set_wrap: true,
                        set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                    },

                    gtk::Button {
                        set_css_classes: &["label-medium", "ok-button-primary"],
                        set_label: "Change Directory",
                        set_halign: gtk::Align::Start,
                        set_hexpand: false,
                        connect_clicked[sender] => move |_| {
                            sender.input(WallpaperSettingsInput::ChangeWallpaperDirectoryClicked);
                        },
                    },

                },

            }
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let mut effects = EffectScope::new();

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let wallpaper_dir = config.wallpaper().wallpaper_dir().get();
            sender_clone.input(WallpaperSettingsInput::WallpaperDirectoryChanged(wallpaper_dir));
        });

        let model = WallpaperSettingsModel {
            wallpaper_directory: "".to_string(),
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
            WallpaperSettingsInput::ChangeWallpaperDirectoryClicked => {
                let dialog = gtk::FileDialog::builder()
                    .title("Choose Wallpaper Directory")
                    .modal(true)
                    .build();

                dialog.select_folder(
                    gtk::Window::NONE,
                    gio::Cancellable::NONE,
                    move |result| {
                        if let Ok(file) = result {
                            if let Some(path) = file.path() {
                                let config_manager = config_manager();
                                config_manager.update_config(|config| {
                                    config.wallpaper.wallpaper_dir = path.to_string_lossy().to_string();
                                });
                            }
                        }
                    },
                );
            }
            WallpaperSettingsInput::WallpaperDirectoryChanged(path) => {
                self.wallpaper_directory = path;
            }
        }

        self.update_view(widgets, sender);
    }
}