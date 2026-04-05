use relm4::{
    gtk,
    ComponentParts,
    ComponentSender,
    SimpleComponent,
};
use relm4::gtk::prelude::WidgetExt;
use okshell_config::schema::menu_widgets::SpacerConfig;

#[derive(Debug, Clone)]
pub(crate) struct SpacerModel {
    size: i32,
}

pub(crate) struct SpacerInit {
    pub config: SpacerConfig,
    pub orientation: gtk::Orientation,
}

#[relm4::component(pub)]
impl SimpleComponent for SpacerModel {
    type Input = ();
    type Output = ();
    type Init = SpacerInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "spacer-menu-widget",
            set_height_request: if params.orientation == gtk::Orientation::Vertical {
                model.size
            } else {
                0
            },
            set_width_request: if params.orientation == gtk::Orientation::Horizontal {
                model.size
            } else {
                0
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SpacerModel {
            size: params.config.size,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}