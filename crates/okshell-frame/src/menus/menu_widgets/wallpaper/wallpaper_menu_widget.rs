use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use gtk4_layer_shell::{KeyboardMode, LayerShell};
use reactive_graph::prelude::{Get, GetUntracked};
use relm4::{gtk, Component, ComponentParts, ComponentSender, RelmWidgetExt};
use relm4::gtk::{gdk, gio, glib, gdk_pixbuf};
use relm4::gtk::prelude::*;
use tracing::info;
use okshell_cache::wallpaper::set_wallpaper;
use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, MatugenStoreFields, ThemeStoreFields, WallpaperStoreFields};
use okshell_config::schema::content_fit::ContentFit;
use okshell_config::schema::themes::{MatugenContrast, MatugenMode, MatugenPreference, MatugenType, Themes};
use okshell_config::schema::wallpaper::ThemeFilterStrength;
use okshell_utils::scroll_extensions::wire_vertical_to_horizontal;
use crate::menus::menu_widgets::wallpaper::parallelogram::ParallelogramPaintable;

fn is_image_file(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| matches!(ext.to_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "webp" | "bmp" | "svg" | "tiff" | "tif"
        ))
}

#[derive(Debug, Clone)]
pub(crate) struct WallpaperMenuWidgetModel {
    is_revealed: bool,
    dir_monitor: Option<gio::FileMonitor>,
    files: Vec<PathBuf>,
    list_store: gio::ListStore,
    thumbnail_width: i32,
    thumbnail_height: i32,
    row_count: u32,
    filter: gtk::CustomFilter,

    wallpaper_directory: String,
    content_fit: ContentFit,

    settings_visible_child: String,

    apply_theme_filter: bool,
    filter_strength: f64,

    matugen_preferences: gtk::StringList,
    active_matugen_preference: MatugenPreference,
    matugen_types: gtk::StringList,
    active_matugen_type: MatugenType,
    matugen_modes: gtk::StringList,
    active_matugen_mode: MatugenMode,
    matugen_contrast: f64,

    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum WallpaperMenuWidgetInput {
    ParentRevealChanged(bool),
    FileAdded(PathBuf),
    FileRemoved(PathBuf),
    FilesUpdated,
    FileClicked(PathBuf),
    SearchFilterChanged(String),
    SearchFilterActivate,

    ChangeWallpaperDirectoryClicked,
    ContentFitChanged(ContentFit),
    ThemeFilterChanged(bool),
    FilterStrengthChanged(f64),

    MatugenPreferenceSelected(MatugenPreference),
    MatugenTypeSelected(MatugenType),
    MatugenModeSelected(MatugenMode),
    MatugenContrastSelected(f64),

    DirectoryEffect(String),
    ContentFitEffect(ContentFit),
    ThemeFilterEffect(bool),
    FilterStrengthEffect(f64),
    ThemeEffect(Themes),

    MatugenTypeEffect(MatugenType),
    MatugenPreferenceEffect(MatugenPreference),
    MatugenModeEffect(MatugenMode),
    MatugenContrastEffect(f64),
}

#[derive(Debug)]
pub(crate) enum WallpaperMenuWidgetOutput {}

pub(crate) struct WallpaperMenuWidgetInit {
    pub thumbnail_width: i32,
    pub thumbnail_height: i32,
    pub row_count: u32,
}

#[derive(Debug)]
pub(crate) enum WallpaperMenuWidgetCommandOutput {}

#[relm4::component(pub)]
impl Component for WallpaperMenuWidgetModel {
    type CommandOutput = WallpaperMenuWidgetCommandOutput;
    type Input = WallpaperMenuWidgetInput;
    type Output = WallpaperMenuWidgetOutput;
    type Init = WallpaperMenuWidgetInit;

    view! {
        #[root]
        #[name = "root"]
        gtk::Box {
            add_css_class: "wallpaper-menu-widget",
            set_orientation: gtk::Orientation::Vertical,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 26,
                set_spacing: 20,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    set_hexpand: false,
                    set_halign: gtk::Align::Start,

                    gtk::Label {
                        add_css_class: "label-xl-bold",
                        set_label: "Wallpaper",
                        set_xalign: 0.0,
                    },

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
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_spacing: 20,

                    gtk::Button {
                        set_css_classes: &["ok-button-primary"],
                        set_halign: gtk::Align::Start,
                        set_hexpand: false,
                        connect_clicked[sender] => move |_| {
                            sender.input(WallpaperMenuWidgetInput::ChangeWallpaperDirectoryClicked);
                        },

                        gtk::Image {
                            set_icon_name: Some("folder-symbolic"),
                        }
                    },

                    #[name = "search_entry"]
                    gtk::Entry {
                        add_css_class: "ok-entry-with-border",
                        set_placeholder_text: Some("Search"),
                        set_hexpand: true,
                        connect_changed[sender] => move |entry| {
                            sender.input(WallpaperMenuWidgetInput::SearchFilterChanged(entry.text().to_string()));
                        },
                        connect_activate[sender] => move |_| {
                            sender.input(WallpaperMenuWidgetInput::SearchFilterActivate);
                        },
                    },

                    gtk::DropDown {
                        set_width_request: 150,
                        set_halign: gtk::Align::Start,
                        set_model: Some(&gtk::StringList::new(&ContentFit::display_names())),
                        #[watch]
                        #[block_signal(handler)]
                        set_selected: model.content_fit.to_index(),
                        connect_selected_notify[sender] => move |dd| {
                            sender.input(WallpaperMenuWidgetInput::ContentFitChanged(
                                ContentFit::from_index(dd.selected())
                            ));
                        } @handler,
                    },
                },

                gtk::Stack {
                    #[watch]
                    set_visible_child_name: model.settings_visible_child.as_str(),
                    set_transition_type: gtk::StackTransitionType::Crossfade,

                    add_named[Some("static")] = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_hexpand: false,
                        set_width_request: 200,
                        set_halign: gtk::Align::Start,
                        set_spacing: 20,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 20,

                            gtk::Label {
                                add_css_class: "label-medium-bold",
                                set_halign: gtk::Align::Start,
                                set_label: "Theme filter",
                                set_hexpand: true,
                            },

                            gtk::Switch {
                                set_valign: gtk::Align::Center,
                                #[watch]
                                #[block_signal(apply_theme_filter_handler)]
                                set_active: model.apply_theme_filter,
                                connect_state_set[sender] => move |_, enabled| {
                                    sender.input(WallpaperMenuWidgetInput::ThemeFilterChanged(enabled));
                                    glib::Propagation::Proceed
                                } @apply_theme_filter_handler,
                            }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 20,

                            gtk::Label {
                                add_css_class: "label-medium-bold",
                                set_halign: gtk::Align::Start,
                                set_label: "Strength",
                                set_hexpand: true,
                            },

                            gtk::SpinButton {
                                set_valign: gtk::Align::Center,
                                set_range: (0.0, 1.0),
                                set_increments: (0.1, 0.1),
                                set_digits: 2,
                                #[watch]
                                #[block_signal(filter_strength_handler)]
                                set_value: model.filter_strength,
                                connect_value_changed[sender] => move |s| {
                                    sender.input(WallpaperMenuWidgetInput::FilterStrengthChanged(s.value()));
                                } @filter_strength_handler,
                            },
                        },
                    },

                    add_named[Some("wallpaper")] = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_hexpand: false,
                        set_width_request: 200,
                        set_halign: gtk::Align::Start,
                        set_spacing: 20,

                        #[name = "matugen_type_dropdown"]
                        gtk::DropDown {
                            set_width_request: 200,
                            set_valign: gtk::Align::Center,
                            set_model: Some(&model.matugen_types),
                            #[watch]
                            #[block_signal(type_handler)]
                            set_selected: MatugenType::all()
                                .iter()
                                .position(|k| k == &model.active_matugen_type)
                                .unwrap_or(0) as u32,
                            connect_selected_notify[sender] => move |dd| {
                                let idx = dd.selected() as usize;
                                if let Some(kind) = MatugenType::all().get(idx) {
                                    sender.input(WallpaperMenuWidgetInput::MatugenTypeSelected(*kind));
                                }
                            } @type_handler,
                        },

                        #[name = "matugen_preference_dropdown"]
                        gtk::DropDown {
                            set_width_request: 200,
                            set_valign: gtk::Align::Center,
                            set_model: Some(&model.matugen_preferences),
                            #[watch]
                            #[block_signal(preference_handler)]
                            set_selected: MatugenPreference::all()
                                .iter()
                                .position(|k| k == &model.active_matugen_preference)
                                .unwrap_or(0) as u32,
                            connect_selected_notify[sender] => move |dd| {
                                let idx = dd.selected() as usize;
                                if let Some(kind) = MatugenPreference::all().get(idx) {
                                    sender.input(WallpaperMenuWidgetInput::MatugenPreferenceSelected(*kind));
                                }
                            } @preference_handler,
                        },

                        #[name = "matugen_mode_dropdown"]
                        gtk::DropDown {
                            set_width_request: 200,
                            set_valign: gtk::Align::Center,
                            set_model: Some(&model.matugen_modes),
                            #[watch]
                            #[block_signal(mode_handler)]
                            set_selected: MatugenMode::all()
                                .iter()
                                .position(|k| k == &model.active_matugen_mode)
                                .unwrap_or(0) as u32,
                            connect_selected_notify[sender] => move |dd| {
                                let idx = dd.selected() as usize;
                                if let Some(kind) = MatugenMode::all().get(idx) {
                                    sender.input(WallpaperMenuWidgetInput::MatugenModeSelected(*kind));
                                }
                            } @mode_handler,
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 20,

                            gtk::Label {
                                add_css_class: "label-medium-bold",
                                set_halign: gtk::Align::Start,
                                set_label: "Contrast",
                                set_hexpand: true,
                            },

                            gtk::SpinButton {
                                set_valign: gtk::Align::Center,
                                set_range: (-1.0, 1.0),
                                set_increments: (0.1, 0.1),
                                set_digits: 2,
                                #[watch]
                                #[block_signal(matugen_contrast_handler)]
                                set_value: model.matugen_contrast,
                                connect_value_changed[sender] => move |s| {
                                    sender.input(WallpaperMenuWidgetInput::MatugenContrastSelected(s.value()));
                                } @matugen_contrast_handler,
                            },
                        },
                    },

                    add_named[Some("none")] = &gtk::Box {}
                },
            },

            gtk::Overlay {
                add_overlay = &gtk::Box {
                    add_css_class: "wallpaper-shadow",
                    set_hexpand: true,
                    set_vexpand: true,
                    set_can_target: false,
                },

                #[name = "scroll_window"]
                gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: false,
                    set_vscrollbar_policy: gtk::PolicyType::Automatic,
                    set_propagate_natural_height: true,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_height_request: (model.thumbnail_height * 2) + 10,

                        gtk::Label {
                            #[watch]
                            set_visible: model.files.is_empty(),
                            set_css_classes: &["wallpaper-empty-message", "label-medium-bold"],
                            set_label: "No wallpapers available",
                        },

                        #[name = "grid_view"]
                        gtk::GridView {
                            set_orientation: gtk::Orientation::Horizontal,
                            #[watch]
                            set_visible: !model.files.is_empty(),
                            set_max_columns: model.row_count,
                            set_min_columns: model.row_count,
                            add_css_class: if model.thumbnail_width < 200 {
                                "wallpaper-grid-small"
                            } else {
                                "wallpaper-grid"
                            },
                        }
                    }
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let list_store = gio::ListStore::new::<gtk::StringObject>();
        let filter = gtk::CustomFilter::new(|_| true);
        let filter_model = gtk::FilterListModel::new(Some(list_store.clone()), Some(filter.clone()));
        let selection = gtk::SingleSelection::new(Some(filter_model.clone()));
        selection.set_autoselect(false);
        selection.set_can_unselect(true);

        let factory = gtk::SignalListItemFactory::new();
        let texture_cache: Arc<Mutex<HashMap<String, gdk::MemoryTexture>>> = Arc::default();

        factory.connect_setup(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let picture = gtk::Picture::new();
            picture.set_content_fit(gtk::ContentFit::Cover);
            picture.set_width_request(params.thumbnail_width);
            picture.add_css_class("wallpaper-thumbnail");
            list_item.set_child(Some(&picture));
        });

        let cache = texture_cache.clone();
        factory.connect_bind(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let string_obj = list_item.item()
                .and_downcast::<gtk::StringObject>()
                .unwrap();
            let path_str = string_obj.string().to_string();
            let picture = list_item.child()
                .and_downcast::<gtk::Picture>()
                .unwrap();

            // Clear any previous texture while we load
            picture.set_paintable(gdk::Paintable::NONE);

            // Store the path we're loading so we can check for staleness
            unsafe { picture.set_data::<String>("loading-path", path_str.clone()) };

            // Check cache first
            {
                let cache = cache.lock().unwrap();
                if let Some(texture) = cache.get(&path_str) {
                    let paintable = ParallelogramPaintable::new(
                        params.thumbnail_width,
                        params.thumbnail_height,
                    );
                    paintable.set_texture(Some(texture.upcast_ref::<gdk::Texture>()));
                    picture.set_paintable(Some(&paintable));
                    return;
                }
            }

            // Spawn thumbnail decode on a background thread.
            let (tx, rx) = std::sync::mpsc::channel::<(String, Option<(glib::Bytes, i32, i32, i32, bool)>)>();

            std::thread::spawn(move || {
                let result = gdk_pixbuf::Pixbuf::from_file_at_scale(
                    &path_str, -1, params.thumbnail_height, true,
                )
                    .ok()
                    .map(|pixbuf: gdk_pixbuf::Pixbuf| {
                        let width = pixbuf.width();
                        let height = pixbuf.height();
                        let rowstride = pixbuf.rowstride();
                        let has_alpha = pixbuf.has_alpha();
                        let bytes = pixbuf.pixel_bytes().unwrap();
                        (bytes, width, height, rowstride, has_alpha)
                    });
                let _ = tx.send((path_str, result));
            });

            // Poll the channel from the main loop
            let cache_insert = cache.clone();
            glib::idle_add_local_once(move || {
                let Ok((path_str, result)) = rx.recv() else { return };

                let still_current = unsafe {
                    picture.data::<String>("loading-path")
                        .map(|p| p.as_ref() == &path_str)
                        .unwrap_or(false)
                };

                if still_current {
                    if let Some((bytes, width, height, rowstride, has_alpha)) = result {
                        let format = if has_alpha {
                            gdk::MemoryFormat::R8g8b8a8
                        } else {
                            gdk::MemoryFormat::R8g8b8
                        };
                        let texture = gdk::MemoryTexture::new(
                            width,
                            height,
                            format,
                            &bytes,
                            rowstride as usize,
                        );

                        cache_insert.lock().unwrap()
                            .insert(path_str, texture.clone());

                        let paintable = ParallelogramPaintable::new(
                            params.thumbnail_width,
                            params.thumbnail_height,
                        );
                        paintable.set_texture(Some(texture.upcast_ref::<gdk::Texture>()));
                        picture.set_paintable(Some(&paintable));
                    }
                }
            });
        });

        factory.connect_unbind(|_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let picture = list_item.child()
                .and_downcast::<gtk::Picture>()
                .unwrap();
            if let Some(paintable) = picture.paintable().and_downcast::<ParallelogramPaintable>() {
                paintable.set_texture(None);
            }
            picture.set_paintable(gdk::Paintable::NONE);
            // Clear the loading-path so any in-flight async load becomes stale
            unsafe { picture.set_data::<String>("loading-path", String::new()) };
        });

        let matugen_preferences = gtk::StringList::new(
            &MatugenPreference::all()
                .iter()
                .map(|p| p.label())
                .collect::<Vec<_>>()
        );

        let matugen_types = gtk::StringList::new(
            &MatugenType::all()
                .iter()
                .map(|p| p.label())
                .collect::<Vec<_>>()
        );

        let matugen_modes = gtk::StringList::new(
            &MatugenMode::all()
                .iter()
                .map(|p| p.label())
                .collect::<Vec<_>>()
        );

        let mut effects = EffectScope::new();
        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let wallpaper_dir = config.wallpaper().wallpaper_dir().get();
            sender_clone.input(WallpaperMenuWidgetInput::DirectoryEffect(wallpaper_dir))
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let value = config_manager().config().wallpaper().content_fit().get();
            sender_clone.input(WallpaperMenuWidgetInput::ContentFitEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let value = config_manager().config().wallpaper().apply_theme_filter().get();
            sender_clone.input(WallpaperMenuWidgetInput::ThemeFilterEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let value = config_manager().config().wallpaper().theme_filter_strength().get();
            sender_clone.input(WallpaperMenuWidgetInput::FilterStrengthEffect(value.get()));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let value = config_manager().config().theme().theme().get();
            sender_clone.input(WallpaperMenuWidgetInput::ThemeEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.theme().matugen().scheme_type().get();
            sender_clone.input(WallpaperMenuWidgetInput::MatugenTypeEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.theme().matugen().preference().get();
            sender_clone.input(WallpaperMenuWidgetInput::MatugenPreferenceEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.theme().matugen().mode().get();
            sender_clone.input(WallpaperMenuWidgetInput::MatugenModeEffect(value));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let value = config.theme().matugen().contrast().get();
            sender_clone.input(WallpaperMenuWidgetInput::MatugenContrastEffect(value.get()));
        });

        let model = WallpaperMenuWidgetModel {
            is_revealed: false,
            dir_monitor: None,
            files: Vec::new(),
            list_store,
            thumbnail_width: params.thumbnail_width,
            thumbnail_height: params.thumbnail_height,
            row_count: params.row_count,
            filter,

            wallpaper_directory: "".to_string(),
            content_fit: config_manager().config().wallpaper().content_fit().get_untracked(),

            settings_visible_child: "none".to_string(),
            apply_theme_filter: config_manager().config().wallpaper().apply_theme_filter().get_untracked(),
            filter_strength: config_manager().config().wallpaper().theme_filter_strength().get_untracked().get(),

            matugen_preferences,
            active_matugen_preference: config_manager().config().theme().matugen().preference().get_untracked(),
            matugen_types,
            active_matugen_type: config_manager().config().theme().matugen().scheme_type().get_untracked(),
            matugen_modes,
            active_matugen_mode: config_manager().config().theme().matugen().mode().get_untracked(),
            matugen_contrast: config_manager().config().theme().matugen().contrast().get_untracked().get(),

            _effects: effects,
        };

        let widgets = view_output!();

        widgets.grid_view.set_model(Some(&selection));
        widgets.grid_view.set_factory(Some(&factory));

        let filter_model_clone = filter_model.clone();
        let sender_clone = sender.clone();
        widgets.grid_view.set_single_click_activate(true);
        widgets.grid_view.connect_activate(move |_, position| {
            if let Some(item) = filter_model_clone.item(position) {
                let string_obj = item.downcast_ref::<gtk::StringObject>().unwrap();
                let path = PathBuf::from(string_obj.string().as_str());
                sender_clone.input(WallpaperMenuWidgetInput::FileClicked(path));
            }
        });

        wire_vertical_to_horizontal(
            &widgets.scroll_window,
            64.0,
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
            WallpaperMenuWidgetInput::ParentRevealChanged(revealed) => {
                // If state is changing from hidden to revealed
                if revealed && !self.is_revealed {
                    if let Some(window) = widgets.root.toplevel_window() {
                        window.set_keyboard_mode(KeyboardMode::OnDemand);
                    }
                    widgets.search_entry.grab_focus();
                // if state is change from revealed to hidden
                } else if !revealed && self.is_revealed {
                    if let Some(window) = widgets.root.toplevel_window() {
                        window.set_keyboard_mode(KeyboardMode::None);
                    }
                    let entry_widget = widgets.search_entry.clone();
                    glib::timeout_add_local_once(
                        std::time::Duration::from_millis(200),
                        move || {
                            entry_widget.set_text("");
                        }
                    );
                }
                self.is_revealed = revealed;
            }
            WallpaperMenuWidgetInput::DirectoryEffect(wallpaper_dir) => {
                self.dir_monitor = None;
                self.files.clear();
                info!("dir changed: {}", wallpaper_dir);

                let path = std::path::Path::new(&wallpaper_dir);
                if path.is_dir() {
                    let dir = gio::File::for_path(path);

                    if let Ok(enumerator) = dir.enumerate_children(
                        gio::FILE_ATTRIBUTE_STANDARD_NAME,
                        gio::FileQueryInfoFlags::NONE,
                        gio::Cancellable::NONE,
                    ) {
                        while let Ok(Some(info)) = enumerator.next_file(gio::Cancellable::NONE) {
                            let child_path = path.join(info.name());
                            if is_image_file(&child_path) {
                                self.files.push(child_path);
                            }
                        }
                    }

                    if let Ok(monitor) = dir.monitor_directory(
                        gio::FileMonitorFlags::NONE,
                        gio::Cancellable::NONE,
                    ) {
                        let sender = sender.clone();
                        monitor.connect_changed(move |_, file, _, event| {
                            let path = file.path().unwrap();
                            match event {
                                gio::FileMonitorEvent::ChangesDoneHint => {
                                    sender.input(WallpaperMenuWidgetInput::FileAdded(path));
                                }
                                gio::FileMonitorEvent::Deleted => {
                                    sender.input(WallpaperMenuWidgetInput::FileRemoved(path));
                                }
                                _ => {}
                            }
                        });
                        self.dir_monitor = Some(monitor);
                    }

                    sender.input(WallpaperMenuWidgetInput::FilesUpdated);
                } else {
                    self.list_store.remove_all();
                }
                self.wallpaper_directory = wallpaper_dir;
            }

            WallpaperMenuWidgetInput::FileAdded(path) => {
                if is_image_file(&path) && !self.files.contains(&path) {
                    self.files.push(path.clone());
                    let path_str = path.to_string_lossy().to_string();
                    let mut insert_pos = self.list_store.n_items();
                    for i in 0..self.list_store.n_items() {
                        if let Some(item) = self.list_store.item(i) {
                            let existing = item.downcast_ref::<gtk::StringObject>().unwrap();
                            if path_str.as_str() < existing.string().as_str() {
                                insert_pos = i;
                                break;
                            }
                        }
                    }
                    let string_obj = gtk::StringObject::new(&path_str);
                    self.list_store.insert(insert_pos, &string_obj);
                }
            }

            WallpaperMenuWidgetInput::FileRemoved(path) => {
                self.files.retain(|p| p != &path);
                let path_str = path.to_string_lossy().to_string();
                for i in 0..self.list_store.n_items() {
                    if let Some(item) = self.list_store.item(i) {
                        let existing = item.downcast_ref::<gtk::StringObject>().unwrap();
                        if existing.string().as_str() == path_str {
                            self.list_store.remove(i);
                            break;
                        }
                    }
                }
            }

            WallpaperMenuWidgetInput::FilesUpdated => {
                self.list_store.remove_all();
                let mut sorted: Vec<_> = self.files.iter().collect();
                sorted.sort();
                for file_path in sorted {
                    let string_obj = gtk::StringObject::new(
                        &file_path.to_string_lossy()
                    );
                    self.list_store.append(&string_obj);
                }
            }
            WallpaperMenuWidgetInput::FileClicked(path) => {
                set_wallpaper(&path);
            }
            WallpaperMenuWidgetInput::ChangeWallpaperDirectoryClicked => {
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
            WallpaperMenuWidgetInput::ContentFitChanged(content_fit) => {
                config_manager().update_config(|config| {
                    config.wallpaper.content_fit = content_fit;
                });
            }
            WallpaperMenuWidgetInput::ThemeFilterChanged(apply) => {
                config_manager().update_config(|config| {
                    config.wallpaper.apply_theme_filter = apply;
                })
            }
            WallpaperMenuWidgetInput::FilterStrengthChanged(strength) => {
                config_manager().update_config(|config| {
                    config.wallpaper.theme_filter_strength = ThemeFilterStrength::new(strength)
                })
            }
            WallpaperMenuWidgetInput::MatugenPreferenceSelected(preference) => {
                config_manager().update_config(|config| {
                    config.theme.matugen.preference = preference;
                });
            }
            WallpaperMenuWidgetInput::MatugenTypeSelected(scheme_type) => {
                config_manager().update_config(|config| {
                    config.theme.matugen.scheme_type = scheme_type;
                });
            }
            WallpaperMenuWidgetInput::MatugenModeSelected(mode) => {
                config_manager().update_config(|config| {
                    config.theme.matugen.mode = mode;
                });
            }
            WallpaperMenuWidgetInput::MatugenContrastSelected(contrast) => {
                config_manager().update_config(|config| {
                    config.theme.matugen.contrast = MatugenContrast::new(contrast);
                });
            }

            WallpaperMenuWidgetInput::ContentFitEffect(content_fit) => {
                self.content_fit = content_fit;
            }
            WallpaperMenuWidgetInput::ThemeFilterEffect(filter) => {
                self.apply_theme_filter = filter;
            }
            WallpaperMenuWidgetInput::FilterStrengthEffect(filter) => {
                self.filter_strength = filter;
            }
            WallpaperMenuWidgetInput::SearchFilterActivate => {

            }
            WallpaperMenuWidgetInput::SearchFilterChanged(text) => {
                let text_lower = text.to_lowercase();
                self.filter.set_filter_func(move |obj| {
                    if text_lower.is_empty() {
                        return true;
                    }
                    let string_obj = obj.downcast_ref::<gtk::StringObject>().unwrap();
                    let path = string_obj.string().to_lowercase();
                    // Match on the filename only, not the full path
                    std::path::Path::new(path.as_str())
                        .file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|name| name.contains(text_lower.as_str()))
                });
            }
            WallpaperMenuWidgetInput::ThemeEffect(theme) => {
                if theme == Themes::Wallpaper {
                    self.settings_visible_child = "wallpaper".to_string();
                } else if theme == Themes::Default {
                    self.settings_visible_child = "none".to_string();
                } else {
                    self.settings_visible_child = "static".to_string();
                }
            }
            WallpaperMenuWidgetInput::MatugenTypeEffect(matugen_type) => {
                self.active_matugen_type = matugen_type;
            }
            WallpaperMenuWidgetInput::MatugenPreferenceEffect(preference) => {
                self.active_matugen_preference = preference;
            }
            WallpaperMenuWidgetInput::MatugenModeEffect(matugen_mode) => {
                self.active_matugen_mode = matugen_mode;
            }
            WallpaperMenuWidgetInput::MatugenContrastEffect(matugen_contrast) => {
                self.matugen_contrast = matugen_contrast;
            }
        }

        self.update_view(widgets, sender);
    }
}