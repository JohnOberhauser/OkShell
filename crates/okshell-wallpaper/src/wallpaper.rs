use std::path::PathBuf;
use gtk4::prelude::{Cast, MediaStreamExt};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use reactive_graph::prelude::Get;
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::gdk;
use relm4::gtk::glib;
use relm4::gtk::prelude::{GtkWindowExt, WidgetExt};
use okshell_cache::wallpaper::{wallpaper_store, WallpaperStateStoreFields};
use okshell_common::scoped_effects::EffectScope;

const TRANSITION_DURATION_MS: u32 = 200;

#[derive(Debug, Clone)]
pub struct WallpaperModel {
    _effects: EffectScope,
}

#[derive(Debug)]
pub enum WallpaperInput {
    PathUpdated(Option<PathBuf>),
}

#[derive(Debug)]
pub enum WallpaperOutput {}

pub struct WallpaperInit {
    pub monitor: gdk::Monitor,
}

#[derive(Debug)]
pub enum WallpaperCommandOutput {}

#[relm4::component(pub)]
impl Component for WallpaperModel {
    type CommandOutput = WallpaperCommandOutput;
    type Input = WallpaperInput;
    type Output = WallpaperOutput;
    type Init = WallpaperInit;

    view! {
        #[root]
        #[name = "root"]
        gtk::Window {
            add_css_class: "wallpaper-window",
            set_decorated: false,
            set_visible: true,

            #[name = "stack"]
            gtk::Stack {
                set_transition_type: gtk::StackTransitionType::Crossfade,
                set_transition_duration: TRANSITION_DURATION_MS,
                set_hexpand: true,
                set_vexpand: true,
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.init_layer_shell();
        root.set_monitor(Some(&params.monitor));
        root.set_namespace(Some("okshell-wallpaper"));
        root.set_layer(Layer::Background);
        root.set_exclusive_zone(-1);
        root.set_anchor(Edge::Top, true);
        root.set_anchor(Edge::Bottom, true);
        root.set_anchor(Edge::Left, true);
        root.set_anchor(Edge::Right, true);

        let mut effects = EffectScope::new();

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let wallpaper = wallpaper_store();
            let path = wallpaper.path().get();
            sender_clone.input(WallpaperInput::PathUpdated(path));
        });

        let model = WallpaperModel {
            _effects: effects,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            WallpaperInput::PathUpdated(path) => {
                if let Some(path) = path {
                    let stack = &widgets.stack;
                    let new_name = path.to_string_lossy().to_string();

                    if let Some(existing) = stack.child_by_name(&new_name) {
                        stack.remove(&existing);
                    }

                    let widget = make_wallpaper_widget(&path);  // ← here

                    let old_child = stack.visible_child();
                    stack.add_named(&widget, Some(&new_name));

                    let stack_clone = stack.clone();
                    glib::idle_add_local_once(move || {
                        stack_clone.set_visible_child_name(&new_name);

                        if let Some(old) = old_child {
                            let stack_clone2 = stack_clone.clone();
                            glib::timeout_add_local_once(
                                std::time::Duration::from_millis(TRANSITION_DURATION_MS as u64 + 50),
                                move || {
                                    // Guard: only remove if still parented to this stack
                                    if old.parent().as_ref() == Some(stack_clone2.upcast_ref()) {
                                        stack_clone2.remove(&old);
                                    }
                                },
                            );
                        }
                    });
                }
            }
        }
    }
}

fn make_wallpaper_widget(path: &std::path::Path) -> gtk::Widget {
    let picture = gtk::Picture::for_filename(&path);
    picture.set_hexpand(true);
    picture.set_vexpand(true);
    picture.set_content_fit(gtk::ContentFit::Cover);
    picture.set_can_shrink(true);
    picture.upcast()
}