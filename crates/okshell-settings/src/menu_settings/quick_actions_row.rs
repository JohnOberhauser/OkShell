use relm4::factory::{DynamicIndex, FactoryComponent};
use relm4::{gtk, FactorySender};
use relm4::gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use okshell_config::schema::menu_widgets::QuickActionWidget;

#[derive(Debug)]
pub struct QuickActionRowModel {
    pub action: QuickActionWidget,
}

#[derive(Debug)]
pub enum QuickActionRowOutput {
    Remove(DynamicIndex),
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
}

#[relm4::factory(pub)]
impl FactoryComponent for QuickActionRowModel {
    type Init = QuickActionWidget;
    type Input = ();
    type Output = QuickActionRowOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 8,
            add_css_class: "quick-action-row",

            gtk::Label {
                set_hexpand: true,
                set_halign: gtk::Align::Start,
                add_css_class: "label-small",
                set_label: self.action.display_name(),
            },
            
            gtk::Button {
                add_css_class: "ok-button-surface",
                set_icon_name: "menu-up-symbolic",
                connect_clicked[sender, index] => move |_| {
                    sender.output(QuickActionRowOutput::MoveUp(index.clone())).unwrap();
                },
            },

            gtk::Button {
                add_css_class: "ok-button-surface",
                set_icon_name: "menu-down-symbolic",
                connect_clicked[sender, index] => move |_| {
                    sender.output(QuickActionRowOutput::MoveDown(index.clone())).unwrap();
                },
            },

            gtk::Button {
                add_css_class: "ok-button-surface",
                set_icon_name: "close-symbolic",
                connect_clicked[sender, index] => move |_| {
                    sender.output(QuickActionRowOutput::Remove(index.clone())).unwrap();
                },
            },
        }
    }

    fn init_model(
        action: Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self { action }
    }
}