use relm4::factory::{DynamicIndex, FactoryComponent};
use relm4::{gtk, FactorySender};
use relm4::gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use okshell_config::schema::bar_widgets::BarWidget;

#[derive(Debug)]
pub struct ActiveWidgetModel {
    pub widget: BarWidget,
}

#[derive(Debug)]
pub enum ActiveWidgetInput {}

#[derive(Debug)]
pub enum ActiveWidgetOutput {
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
    Remove(DynamicIndex),
}

#[relm4::factory(pub)]
impl FactoryComponent for ActiveWidgetModel {
    type Init = BarWidget;
    type Input = ActiveWidgetInput;
    type Output = ActiveWidgetOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 8,
            
            gtk::Label {
                add_css_class: "label-small",
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                #[watch]
                set_label: self.widget.display_name().to_string().as_str(),
            },

            gtk::Button {
                add_css_class: "ok-button-surface",
                set_icon_name: "menu-up-symbolic",
                connect_clicked[sender, index] => move |_| {
                    sender.output(ActiveWidgetOutput::MoveUp(index.clone())).unwrap();
                },
            },

            gtk::Button {
                add_css_class: "ok-button-surface",
                set_icon_name: "menu-down-symbolic",
                connect_clicked[sender, index] => move |_| {
                    sender.output(ActiveWidgetOutput::MoveDown(index.clone())).unwrap();
                },
            },

            gtk::Button {
                add_css_class: "ok-button-surface",
                set_icon_name: "close-symbolic",
                connect_clicked[sender, index] => move |_| {
                    sender.output(ActiveWidgetOutput::Remove(index.clone())).unwrap();
                },
            },
        }
    }

    fn init_model(widget: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { widget }
    }
}