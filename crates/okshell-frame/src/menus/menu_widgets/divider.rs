use relm4::{
    gtk,
    ComponentParts,
    ComponentSender,
    SimpleComponent,
};
use relm4::gtk::prelude::{OrientableExt, WidgetExt};

#[derive(Debug, Clone)]
pub(crate) struct DividerMenuWidgetModel {}

#[derive(Debug)]
pub(crate) enum DividerMenuWidgetInput {}

#[derive(Debug)]
pub(crate) enum DividerMenuWidgetOutput {}

pub(crate) struct DividerMenuWidgetInit {
    pub orientation: gtk::Orientation,
}

#[relm4::component(pub)]
impl SimpleComponent for DividerMenuWidgetModel {
    type Input = DividerMenuWidgetInput;
    type Output = DividerMenuWidgetOutput;
    type Init = DividerMenuWidgetInit;

    view! {
        #[root]
        gtk::Separator {
            add_css_class: "divider-menu-widget",
            set_orientation: if params.orientation == gtk::Orientation::Vertical {
                gtk::Orientation::Horizontal
            } else {
                gtk::Orientation::Vertical
            },
            set_margin_top: 20,
            set_margin_bottom: 20,
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = DividerMenuWidgetModel {};

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}