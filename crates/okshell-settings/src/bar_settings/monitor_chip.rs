use relm4::factory::{DynamicIndex, FactoryComponent};
use relm4::gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::{FactorySender, gtk};

#[derive(Debug)]
pub struct MonitorChipModel {
    pub name: String,
}

#[derive(Debug)]
pub enum MonitorChipOutput {
    Remove(DynamicIndex),
}

#[relm4::factory(pub)]
impl FactoryComponent for MonitorChipModel {
    type Init = String;
    type Input = ();
    type Output = MonitorChipOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "monitor-chip",
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 8,
            set_hexpand: false,

            gtk::Label {
                add_css_class: "label-small-primary",
                set_label: &self.name,
                set_xalign: 0.0,
                set_hexpand: true,
            },

            gtk::Button {
                add_css_class: "ok-button-primary",
                connect_clicked[sender, index] => move |_| {
                    sender.output(MonitorChipOutput::Remove(index.clone())).unwrap();
                },
                set_icon_name: "close-symbolic",
            },
        }
    }

    fn init_model(name: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { name }
    }
}
