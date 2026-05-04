use relm4::gtk::prelude::WidgetExt;
use relm4::{Component, ComponentParts, ComponentSender, gtk};

#[derive(Debug, Clone)]
pub(crate) struct VpnIndicatorModel {}

#[derive(Debug)]
pub(crate) enum VpnIndicatorInput {}

#[derive(Debug)]
pub(crate) enum VpnIndicatorOutput {}

pub(crate) struct VpnIndicatorInit {}

#[derive(Debug)]
pub(crate) enum VpnIndicatorCommandOutput {}

#[relm4::component(pub)]
impl Component for VpnIndicatorModel {
    type CommandOutput = VpnIndicatorCommandOutput;
    type Input = VpnIndicatorInput;
    type Output = VpnIndicatorOutput;
    type Init = VpnIndicatorInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "vpn-indicator-bar-widget",
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = VpnIndicatorModel {};

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {}
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {}
    }
}
