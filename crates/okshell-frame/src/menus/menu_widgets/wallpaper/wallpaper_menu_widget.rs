use std::path::PathBuf;
use reactive_graph::prelude::Get;
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::{gdk, gio, glib, gdk_pixbuf};
use relm4::gtk::prelude::*;
use tracing::info;
use okshell_cache::wallpaper::set_wallpaper;
use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, WallpaperStoreFields};
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
    dir_monitor: Option<gio::FileMonitor>,
    files: Vec<PathBuf>,
    list_store: gio::ListStore,
    thumbnail_width: i32,
    thumbnail_height: i32,
    row_count: u32,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum WallpaperMenuWidgetInput {
    DirectoryChanged(String),
    FileAdded(PathBuf),
    FileRemoved(PathBuf),
    FilesUpdated,
    FileClicked(PathBuf),
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
        gtk::Overlay {
            add_css_class: "wallpaper-menu-widget",

            add_overlay = &gtk::Box {
                add_css_class: "wallpaper-shadow",
                set_hexpand: true,
                set_vexpand: true,
                set_can_target: false,
            },

            gtk::Box {

                #[name = "scroll_window"]
                gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: false,
                    set_vscrollbar_policy: gtk::PolicyType::Automatic,
                    set_propagate_natural_height: true,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

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

        let factory = gtk::SignalListItemFactory::new();

        factory.connect_setup(move |_, list_item| {
            let list_item = list_item.downcast_ref::<gtk::ListItem>().unwrap();
            let picture = gtk::Picture::new();
            picture.set_content_fit(gtk::ContentFit::Cover);
            picture.set_width_request(params.thumbnail_width);
            picture.add_css_class("wallpaper-thumbnail");
            list_item.set_child(Some(&picture));
        });

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

            // Spawn thumbnail decode on a background thread.
            // Pixbuf isn't Send, so we extract the raw bytes and dimensions
            // on the background thread and send them back via a channel.
            // The weak ref stays on the main thread side.
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

        let selection = gtk::SingleSelection::new(Some(list_store.clone()));
        selection.set_autoselect(false);
        selection.set_can_unselect(true);

        let mut effects = EffectScope::new();
        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config_manager().config();
            let wallpaper_dir = config.wallpaper().wallpaper_dir().get();
            sender_clone.input(WallpaperMenuWidgetInput::DirectoryChanged(wallpaper_dir))
        });

        let model = WallpaperMenuWidgetModel {
            dir_monitor: None,
            files: Vec::new(),
            list_store,
            thumbnail_width: params.thumbnail_width,
            thumbnail_height: params.thumbnail_height,
            row_count: params.row_count,
            _effects: effects,
        };

        let widgets = view_output!();

        widgets.grid_view.set_model(Some(&selection));
        widgets.grid_view.set_factory(Some(&factory));

        let list_store_clone = model.list_store.clone();
        let sender_clone = sender.clone();
        widgets.grid_view.set_single_click_activate(true);
        widgets.grid_view.connect_activate(move |_, position| {
            if let Some(item) = list_store_clone.item(position) {
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
            WallpaperMenuWidgetInput::DirectoryChanged(wallpaper_dir) => {
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
                set_wallpaper(path);
            }
        }

        self.update_view(widgets, sender);
    }
}