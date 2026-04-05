use relm4::{
    gtk,
    ComponentParts,
    ComponentSender,
    SimpleComponent,
};
use relm4::gtk::prelude::{BoxExt, OrientableExt, WidgetExt};
use okshell_common::dynamic_box::generic_widget_controller::GenericWidgetController;
use okshell_config::schema::menu_widgets::ContainerConfig;
use okshell_config::schema::position::Orientation;
use crate::menus::builder::build_widget;
use crate::menus::menu::MenuModel;

pub(crate) struct ContainerModel {
    widget_controllers: Vec<Box<dyn GenericWidgetController>>,
    spacing: i32,
    orientation: gtk::Orientation,
    minimum_width: i32,
}

#[derive(Debug)]
pub(crate) enum ContainerInput {}

#[derive(Debug)]
pub(crate) enum ContainerOutput {}

pub(crate) struct ContainerInit {
    pub config: ContainerConfig,
    pub menu_sender: ComponentSender<MenuModel>,
}

#[relm4::component(pub)]
impl SimpleComponent for ContainerModel {
    type Input = ContainerInput;
    type Output = ContainerOutput;
    type Init = ContainerInit;

    view! {
        #[root]
        #[name = "widget_container"]
        gtk::Box {
            add_css_class: "container-menu-widget",
            set_orientation: model.orientation,
            set_hexpand: true,
            set_spacing: model.spacing,
            set_width_request: model.minimum_width,
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let mut widget_controllers: Vec<Box<dyn GenericWidgetController>> = Vec::new();

        for item in params.config.widgets {
            let controller = build_widget(
                &item,
                if params.config.orientation == Orientation::Horizontal {
                    gtk::Orientation::Horizontal
                } else {
                    gtk::Orientation::Vertical
                },
                &params.menu_sender,
            );
            widget_controllers.push(controller);
        }

        let model = ContainerModel {
            widget_controllers,
            spacing: params.config.spacing,
            orientation: if params.config.orientation == Orientation::Horizontal {
                gtk::Orientation::Horizontal
            } else {
                gtk::Orientation::Vertical
            },
            minimum_width: params.config.minimum_width,
        };

        let widgets = view_output!();

        for controller in &model.widget_controllers {
            widgets.widget_container.append(&controller.root_widget());
        }

        ComponentParts { model, widgets }
    }
}