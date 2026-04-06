use std::sync::Arc;
use relm4::{gtk, gtk::gdk, gtk::glib, Component, ComponentParts, ComponentSender, Sender};
use relm4::gtk::prelude::*;
use wayle_systray::adapters::gtk4::Adapter;
use wayle_systray::core::item::TrayItem;
use wayle_systray::types::item::IconPixmap;

#[derive(Debug, Clone)]
pub(crate) struct SystemTrayItemModel {
    tray_item: Arc<TrayItem>,
    popover: Option<gtk::PopoverMenu>,
}

#[derive(Debug)]
pub(crate) enum SystemTrayItemInput {
    Clicked,
}

#[derive(Debug)]
pub(crate) enum SystemTrayItemOutput {}

#[relm4::component(pub)]
impl Component for SystemTrayItemModel {
    type CommandOutput = ();
    type Input = SystemTrayItemInput;
    type Output = SystemTrayItemOutput;
    type Init = Arc<TrayItem>;

    view! {
        #[root]
        gtk::Box {
            #[name = "button"]
            gtk::Button {
                set_css_classes: &["ok-button-surface", "ok-bar-widget"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(SystemTrayItemInput::Clicked);
                },
                add_controller = gtk::GestureClick::builder().button(3).build() {
                    connect_released[sender] => move |_, _, _, _| {
                        sender.input(SystemTrayItemInput::Clicked);
                    },
                },

                #[name="image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                },
            },
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SystemTrayItemModel {
            tray_item: params,
            popover: None,
        };

        let widgets = view_output!();

        Self::update_icon(&model, &widgets.image);

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root
    ) {
        match message {
            SystemTrayItemInput::Clicked => {
                // Remove old popover if it exists
                if let Some(old) = self.popover.take() {
                    old.unparent();
                }

                let model = Adapter::build_model(&self.tray_item);

                let popover = gtk::PopoverMenu::from_model_full(
                    &model.menu,
                    gtk::PopoverMenuFlags::NESTED,
                );

                popover.insert_action_group("app", Some(&model.actions));
                popover.set_parent(&widgets.button);
                popover.popup();
                self.popover = Some(popover);
            }
        }
    }

    fn shutdown(
        &mut self,
        _widgets: &mut Self::Widgets,
        _output: Sender<Self::Output>
    ) {
        if let Some(popover) = self.popover.take() {
            popover.unparent();
        }
    }
}

impl SystemTrayItemModel {
    // Logic comes from wayle shell
    // https://github.com/Jas-SinghFSU/wayle/blob/master/crates/wayle-shell/src/shell/bar/modules/systray/item/mod.rs
    fn update_icon(
        &self,
        image: &gtk::Image,
    ) {
        if let Some(icon_name) = self.tray_item.icon_name.get() {
            let theme_path = self.tray_item.icon_theme_path.get();
            if let Some(texture) = theme_path
                .as_deref()
                .and_then(|p| Self::load_icon_from_theme_path(p, &icon_name))
            {
                image.set_paintable(Some(&texture));
                return;
            }
            image.set_icon_name(Some(&icon_name));
            return;
        }

        if let Some(texture) = Self::select_best_pixmap(
            &self.tray_item.icon_pixmap.get(),
            24
        ).and_then(Self::create_texture_from_pixmap) {
            image.set_paintable(Some(&texture));
            return;
        }

        image.set_icon_name(Some("application-x-executable-symbolic"));
    }

    fn select_best_pixmap(
        pixmaps: &[IconPixmap],
        target_size: i32,
    ) -> Option<&IconPixmap> {
        pixmaps
            .iter()
            .min_by_key(|p| (p.width - target_size).abs() + (p.height - target_size).abs())
    }

    fn create_texture_from_pixmap(pixmap: &IconPixmap) -> Option<gdk::Texture> {
        let rgba_data = Self::argb_to_rgba(&pixmap.data);
        let bytes = glib::Bytes::from_owned(rgba_data);

        gdk::MemoryTexture::new(
            pixmap.width,
            pixmap.height,
            gdk::MemoryFormat::R8g8b8a8,
            &bytes,
            (pixmap.width * 4) as usize,
        )
            .upcast::<gdk::Texture>()
            .into()
    }

    fn argb_to_rgba(argb: &[u8]) -> Vec<u8> {
        argb.chunks_exact(4)
            .flat_map(|chunk| {
                let a = chunk[0];
                let r = chunk[1];
                let g = chunk[2];
                let b = chunk[3];
                [r, g, b, a]
            })
            .collect()
    }

    fn load_icon_from_theme_path(theme_path: &str, icon_name: &str) -> Option<gdk::Texture> {
        if theme_path.is_empty() {
            return None;
        }

        for ext in ["png", "svg", "xpm"] {
            let file_path = format!("{theme_path}/{icon_name}.{ext}");
            if let Ok(texture) = gdk::Texture::from_filename(&file_path) {
                return Some(texture);
            }
        }

        None
    }
}