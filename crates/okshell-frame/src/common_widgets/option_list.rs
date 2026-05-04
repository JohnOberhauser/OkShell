use relm4::gtk;
use relm4::gtk::pango;
use relm4::gtk::prelude::*;
use relm4::prelude::*;
use std::fmt::Debug;

pub trait OptionItem: Debug + Clone + Send + 'static {
    fn label(&self) -> String;
    fn icon_name(&self) -> Option<String>;
}

pub struct OptionsList<T: OptionItem> {
    pub options: Vec<T>,
}

#[derive(Debug)]
pub enum OptionsListInput<T: Debug> {
    SetOptions(Vec<T>),
}

#[derive(Debug)]
pub enum OptionsListOutput<T: Debug> {
    Selected(T),
}

#[relm4::component(pub)]
impl<T: OptionItem> Component for OptionsList<T> {
    type Init = Vec<T>;
    type Input = OptionsListInput<T>;
    type Output = OptionsListOutput<T>;
    type CommandOutput = ();

    view! {
        #[name = "container"]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
        }
    }

    fn init(
        options: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = OptionsList {
            options: options.clone(),
        };
        let widgets = view_output!();
        populate(&widgets.container, &options, &sender);
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
            OptionsListInput::SetOptions(new_options) => {
                self.options = new_options;
                while let Some(child) = widgets.container.first_child() {
                    widgets.container.remove(&child);
                }
                populate(&widgets.container, &self.options, &sender);
            }
        }

        self.update_view(widgets, sender);
    }
}

fn populate<T: OptionItem>(
    container: &gtk::Box,
    options: &[T],
    sender: &ComponentSender<OptionsList<T>>,
) {
    for item in options {
        let btn = gtk::Button::new();
        btn.set_css_classes(&["ok-button-surface"]);
        btn.set_hexpand(true);

        let content_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);

        if let Some(icon) = item.icon_name() {
            let image = gtk::Image::from_icon_name(&icon);
            content_box.append(&image);
        }

        let label = gtk::Label::new(Some(&item.label()));
        label.set_halign(gtk::Align::Start);
        label.set_hexpand(true);
        label.set_ellipsize(pango::EllipsizeMode::End);
        content_box.append(&label);

        btn.set_child(Some(&content_box));

        let item_clone = item.clone();
        let sender = sender.clone();
        btn.connect_clicked(move |_| {
            sender
                .output(OptionsListOutput::Selected(item_clone.clone()))
                .unwrap_or_default();
        });

        container.append(&btn);
    }
}
