use gtk4::gdk;
use gtk4::prelude::{BoxExt, GtkWindowExt, OrientableExt, RangeExt, WidgetExt};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use okshell_common::WatcherToken;
use okshell_services::brightness_service;
use okshell_utils::brightness::{get_brightness_icon, spawn_brightness_watcher};
use relm4::{Component, ComponentParts, ComponentSender, gtk};

#[derive(Debug)]
pub struct BrightnessOsdModel {
    hide_token: WatcherToken,
    icon_name: String,
    slider_value: f64,
    shown_count: u16,
}

#[derive(Debug)]
pub enum BrightnessOsdInput {
    BrightnessChanged,
    Show,
    Hide,
}

#[derive(Debug)]
pub enum BrightnessOsdOutput {}

pub struct BrightnessOsdInit {
    pub monitor: gdk::Monitor,
}

#[derive(Debug)]
pub enum BrightnessOsdCommandOutput {
    BrightnessChanged,
    Hide,
}

#[relm4::component(pub)]
impl Component for BrightnessOsdModel {
    type CommandOutput = BrightnessOsdCommandOutput;
    type Input = BrightnessOsdInput;
    type Output = BrightnessOsdOutput;
    type Init = BrightnessOsdInit;

    view! {
        #[root]
        gtk::Window {
            set_css_classes: &["osd-window", "window-opacity"],
            set_decorated: false,
            set_visible: false,
            set_default_height: 1,
            set_margin_bottom: 200,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_width_request: 300,
                set_spacing: 20,

                gtk::Image {
                    add_css_class: "osd-icon",
                    #[watch]
                    set_icon_name: Some(model.icon_name.as_str()),
                },

                gtk::Scale {
                    add_css_class: "ok-progress-bar",
                    set_hexpand: true,
                    set_can_focus: false,
                    set_focus_on_click: false,
                    set_range: (0.0, 1.0),
                    #[watch]
                    set_value: model.slider_value,
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.init_layer_shell();
        root.set_monitor(Some(&params.monitor));
        root.set_namespace(Some("okshell-osd"));
        root.set_layer(Layer::Overlay);
        root.set_exclusive_zone(0);
        root.set_anchor(Edge::Bottom, true);

        spawn_brightness_watcher(&sender, || BrightnessOsdCommandOutput::BrightnessChanged);

        let model = BrightnessOsdModel {
            hide_token: WatcherToken::new(),
            icon_name: "brightness-medium-symbolic".to_string(),
            slider_value: 0.0,
            shown_count: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            BrightnessOsdInput::BrightnessChanged => {
                if let Some(service) = brightness_service() {
                    let device = service.primary.get();

                    if let Some(device) = device {
                        let brightness = device.percentage();

                        self.icon_name = get_brightness_icon(brightness.value()).to_string();
                        self.slider_value = brightness.fraction();

                        let token = self.hide_token.reset();
                        sender.command(|out, shutdown| {
                            shutdown
                                .register(async move {
                                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                                    if !token.is_cancelled() {
                                        out.send(BrightnessOsdCommandOutput::Hide).ok();
                                    }
                                })
                                .drop_on_shutdown()
                        });
                    }
                }
            }
            BrightnessOsdInput::Show => {
                if self.shown_count > 1 {
                    root.set_visible(true);
                } else {
                    self.shown_count = self.shown_count + 1;
                }
            }
            BrightnessOsdInput::Hide => {
                root.set_visible(false);
            }
        }

        self.update_view(widgets, sender);
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            BrightnessOsdCommandOutput::BrightnessChanged => {
                sender.input(BrightnessOsdInput::BrightnessChanged);
                sender.input(BrightnessOsdInput::Show);
            }
            BrightnessOsdCommandOutput::Hide => {
                sender.input(BrightnessOsdInput::Hide);
            }
        }
    }
}
