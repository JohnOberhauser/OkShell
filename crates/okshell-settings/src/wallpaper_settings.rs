use reactive_graph::prelude::{Get, GetUntracked};
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::gio;
use relm4::gtk::prelude::{BoxExt, ButtonExt, FileExt, OrientableExt, WidgetExt};
use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, WallpaperStoreFields};
use okshell_config::schema::content_fit::ContentFit;

#[derive(Debug, Clone)]
pub(crate) struct WallpaperSettingsModel {
    wallpaper_directory: String,
    content_fit: ContentFit,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum WallpaperSettingsInput {
    ChangeWallpaperDirectoryClicked,
    ContentFitChanged(ContentFit),
    
    WallpaperDirectoryEffect(String),
    ContentFitEffect(ContentFit),
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
                
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Label {
                            add_css_class: "label-medium-bold",
                            set_halign: gtk::Align::Start,
                            set_label: "Content fit",
                            set_hexpand: true,
                        },

                        gtk::Label {
                            add_css_class: "label-small",
                            set_halign: gtk::Align::Start,
                            set_label: "How the wallpaper should fit into the space.",
                            set_hexpand: true,
                            set_xalign: 0.0,
                            set_wrap: true,
                            set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 150,
                        set_valign: gtk::Align::Center,
                        set_model: Some(&gtk::StringList::new(&ContentFit::display_names())),
                        #[watch]
                        #[block_signal(handler)]
                        set_selected: model.content_fit.to_index(),
                        connect_selected_notify[sender] => move |dd| {
                            sender.input(WallpaperSettingsInput::ContentFitChanged(
                                ContentFit::from_index(dd.selected())
                            ));
                        } @handler,
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
            let wallpaper_dir = config_manager().config().wallpaper().wallpaper_dir().get();
            sender_clone.input(WallpaperSettingsInput::WallpaperDirectoryEffect(wallpaper_dir));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let value = config_manager().config().wallpaper().content_fit().get();
            sender_clone.input(WallpaperSettingsInput::ContentFitEffect(value));
        });

        let model = WallpaperSettingsModel {
            wallpaper_directory: "".to_string(),
            content_fit: config_manager().config().wallpaper().content_fit().get_untracked(),
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
                                config_manager().update_config(|config| {
                                    config.wallpaper.wallpaper_dir = path.to_string_lossy().to_string();
                                });
                            }
                        }
                    },
                );
            }
            WallpaperSettingsInput::ContentFitChanged(content_fit) => {
                config_manager().update_config(|config| {
                    config.wallpaper.content_fit = content_fit;
                });
            }
            
            WallpaperSettingsInput::WallpaperDirectoryEffect(path) => {
                self.wallpaper_directory = path;
            }
            WallpaperSettingsInput::ContentFitEffect(content_fit) => {
                self.content_fit = content_fit;
            }
        }

        self.update_view(widgets, sender);
    }
}