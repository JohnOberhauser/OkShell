use okshell_clipboard::{ClipboardEntry, EntryPreview, clipboard_service};
use relm4::{
    Component, ComponentParts, ComponentSender, RelmWidgetExt,
    gtk::{self, glib, prelude::*},
};

#[derive(Debug, Clone)]
pub(crate) struct ClipboardItemModel {
    entry: ClipboardEntry,
}

#[derive(Debug)]
pub(crate) enum ClipboardItemInput {
    DeleteEntry,
    CopyEntry,
}

#[derive(Debug)]
pub(crate) enum ClipboardItemOutput {}

#[derive(Debug)]
pub(crate) enum ClipboardItemCommandOutput {}

#[relm4::component(pub)]
impl Component for ClipboardItemModel {
    type CommandOutput = ClipboardItemCommandOutput;
    type Input = ClipboardItemInput;
    type Output = ClipboardItemOutput;
    type Init = ClipboardEntry;

    view! {
        #[root]
        gtk::Overlay {
            add_overlay = &gtk::Button {
                add_css_class: "ok-button-surface",
                add_css_class: "clipboard-trash-button",
                set_halign: gtk::Align::End,
                set_valign: gtk::Align::Start,
                set_hexpand: false,
                set_vexpand: false,
                set_margin_all: 8,
                connect_clicked[sender] => move |_| {
                    let _ = sender.input(ClipboardItemInput::DeleteEntry);
                },

                #[name="image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("trash-symbolic"),
                },
            },

            gtk::Button {
                add_css_class: "clipboard-copy-button",
                connect_clicked[sender] => move |_| {
                    let _ = sender.input(ClipboardItemInput::CopyEntry);
                },

                gtk::Box {
                    add_css_class: "clipboard-item",
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 12,

                    gtk::Label {
                        add_css_class: "clipboard-item-title",
                        set_label: format!("#{}", model.entry.id.to_string()).as_str(),
                        set_hexpand: true,
                        set_halign: gtk::Align::Start,
                    },

                    // Preview — conditionally built in init
                    #[name = "preview_box"]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_valign: gtk::Align::Center,
                    },
                },
            },
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ClipboardItemModel { entry: params };
        let widgets = view_output!();

        // Build the preview content based on entry type.
        match &model.entry.preview {
            EntryPreview::Text(text) => {
                let label = gtk::Label::builder()
                    .label(text)
                    .halign(gtk::Align::Start)
                    .ellipsize(gtk::pango::EllipsizeMode::End)
                    .max_width_chars(60)
                    .lines(2)
                    .wrap(true)
                    .wrap_mode(gtk::pango::WrapMode::WordChar)
                    .build();
                label.add_css_class("label-small-bold");
                widgets.preview_box.append(&label);
            }
            EntryPreview::Image {
                rgba,
                width,
                height,
            } => {
                let bytes = glib::Bytes::from(rgba);
                let texture = gtk::gdk::MemoryTexture::new(
                    *width as i32,
                    *height as i32,
                    gtk::gdk::MemoryFormat::R8g8b8a8,
                    &bytes,
                    (*width * 4) as usize,
                );
                let picture = gtk::Picture::for_paintable(&texture);
                picture.set_content_fit(gtk::ContentFit::Cover);
                picture.set_hexpand(true);

                let frame = gtk::Box::new(gtk::Orientation::Vertical, 0);
                frame.set_overflow(gtk::Overflow::Hidden);
                frame.set_height_request(200);
                frame.set_hexpand(true);
                frame.append(&picture);
                frame.add_css_class("clipboard-item-image");
                widgets.preview_box.append(&frame);
            }
            EntryPreview::Binary { mime_type, size } => {
                let label = gtk::Label::builder()
                    .label(&format!("{mime_type}  ({})", format_size(*size)))
                    .halign(gtk::Align::Start)
                    .build();
                label.add_css_class("label-small-bold");
                widgets.preview_box.append(&label);
            }
        }

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
            ClipboardItemInput::DeleteEntry => {
                clipboard_service().delete_entry(self.entry.id);
            }
            ClipboardItemInput::CopyEntry => {
                clipboard_service().copy_entry(self.entry.id);
            }
        }
        self.update_view(widgets, sender);
    }
}

fn format_size(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
