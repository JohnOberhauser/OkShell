use okshell_gamma::{GammaState, gamma_service};
use relm4::gtk::glib;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use relm4::{Component, ComponentParts, ComponentSender, gtk};

#[derive(Debug, Clone)]
pub(crate) struct NightLightModel {
    enabled: bool,
}

#[derive(Debug)]
pub(crate) enum NightLightInput {
    Clicked,
    GammaStateChanged(GammaState),
}

#[derive(Debug)]
pub(crate) enum NightLightOutput {}

pub(crate) struct NightLightInit {}

#[derive(Debug)]
pub(crate) enum NightLightCommandOutput {}

#[relm4::component(pub)]
impl Component for NightLightModel {
    type CommandOutput = NightLightCommandOutput;
    type Input = NightLightInput;
    type Output = NightLightOutput;
    type Init = NightLightInit;

    view! {
        #[root]
        gtk::Box {
            #[name = "button"]
            gtk::Button {
                #[watch]
                set_css_classes: if model.enabled {
                    &["ok-button-surface", "ok-button-medium", "selected"]
                } else {
                    &["ok-button-surface", "ok-button-medium"]
                },
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(NightLightInput::Clicked);
                },

                #[name = "action_icon_image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("nightlight-symbolic"),
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let state = gamma_service().state();

        let sender_clone = sender.clone();
        let mut rx = gamma_service().subscribe();
        glib::spawn_future_local(async move {
            while rx.changed().await.is_ok() {
                let state = rx.borrow_and_update().clone();
                sender_clone.input(NightLightInput::GammaStateChanged(state));
            }
        });

        let model = NightLightModel {
            enabled: state.enabled,
        };

        let widgets = view_output!();

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
            NightLightInput::Clicked => {
                let gamma_service = gamma_service();
                gamma_service.set_enabled(!self.enabled);
            }
            NightLightInput::GammaStateChanged(state) => {
                self.enabled = state.enabled;
            }
        }

        self.update_view(widgets, sender);
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
