use gtk4::prelude::Cast;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use okshell_cache::wallpaper::{
    WallpaperImage, WallpaperStateStoreFields, current_wallpaper_image, wallpaper_store,
};
use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, WallpaperStoreFields};
use okshell_config::schema::content_fit::ContentFit;
use reactive_graph::prelude::{Get, GetUntracked};
use relm4::gtk::gdk;
use relm4::gtk::glib;
use relm4::gtk::prelude::{GtkWindowExt, WidgetExt};
use relm4::{Component, ComponentParts, ComponentSender, gtk};

const TRANSITION_DURATION_MS: u32 = 200;

#[derive(Debug, Clone)]
pub struct WallpaperModel {
    content_fit: ContentFit,
    _effects: EffectScope,
}

#[derive(Debug)]
pub enum WallpaperInput {
    WallpaperChanged(u64),
    ContentFitChanged(ContentFit),
}

#[derive(Debug)]
pub enum WallpaperOutput {}

pub struct WallpaperInit {
    pub monitor: gdk::Monitor,
}

#[relm4::component(pub)]
impl Component for WallpaperModel {
    type CommandOutput = ();
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
            let revision = wallpaper_store().revision().get();
            sender_clone.input(WallpaperInput::WallpaperChanged(revision));
        });

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let value = config_manager().config().wallpaper().content_fit().get();
            sender_clone.input(WallpaperInput::ContentFitChanged(value));
        });

        let model = WallpaperModel {
            content_fit: config_manager()
                .config()
                .wallpaper()
                .content_fit()
                .get_untracked(),
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
            WallpaperInput::WallpaperChanged(revision) => {
                let Some(image) = current_wallpaper_image() else {
                    while let Some(child) = widgets.stack.first_child() {
                        widgets.stack.remove(&child);
                    }
                    return;
                };

                let name = format!("wallpaper-{revision}");
                let stack = &widgets.stack;

                let widget = make_wallpaper_widget(&image, gtk_content_fit(&self.content_fit));
                let old_child = stack.visible_child();
                stack.add_named(&widget, Some(&name));
                transition_stack(stack, &name, old_child);
            }
            WallpaperInput::ContentFitChanged(content_fit) => {
                self.content_fit = content_fit;
                let fit = gtk_content_fit(&self.content_fit);
                let mut child = widgets.stack.first_child();
                while let Some(widget) = child {
                    child = widget.next_sibling();
                    if let Some(picture) = widget.downcast_ref::<gtk::Picture>() {
                        picture.set_content_fit(fit);
                    }
                }
            }
        }
    }
}

fn transition_stack(stack: &gtk::Stack, new_name: &str, old_child: Option<gtk::Widget>) {
    let stack_clone = stack.clone();
    let new_name = new_name.to_string();
    glib::idle_add_local_once(move || {
        stack_clone.set_visible_child_name(&new_name);

        if let Some(old) = old_child {
            let stack_clone2 = stack_clone.clone();
            glib::timeout_add_local_once(
                std::time::Duration::from_millis(TRANSITION_DURATION_MS as u64 + 50),
                move || {
                    if old.parent().as_ref() == Some(stack_clone2.upcast_ref()) {
                        stack_clone2.remove(&old);
                    }
                },
            );
        }
    });
}

fn make_wallpaper_widget(image: &WallpaperImage, content_fit: gtk::ContentFit) -> gtk::Widget {
    let bytes = glib::Bytes::from(&*image.buf);
    let texture = gdk::MemoryTexture::new(
        image.width as i32,
        image.height as i32,
        gdk::MemoryFormat::R8g8b8a8,
        &bytes,
        (image.width * 4) as usize,
    );

    let picture = gtk::Picture::for_paintable(&texture);
    picture.set_hexpand(true);
    picture.set_vexpand(true);
    picture.set_content_fit(content_fit);
    picture.set_can_shrink(true);
    picture.upcast()
}

fn gtk_content_fit(content_fit: &ContentFit) -> gtk::ContentFit {
    match content_fit {
        ContentFit::Contain => gtk::ContentFit::Contain,
        ContentFit::Cover => gtk::ContentFit::Cover,
        ContentFit::Fill => gtk::ContentFit::Fill,
        ContentFit::ScaleDown => gtk::ContentFit::ScaleDown,
    }
}
