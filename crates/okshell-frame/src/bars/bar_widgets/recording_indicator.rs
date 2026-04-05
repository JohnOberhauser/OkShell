use reactive_graph::traits::{Get, GetUntracked};
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::gtk::Orientation;
use relm4::gtk::prelude::{ButtonExt, WidgetExt};
use okshell_common::scoped_effects::EffectScope;
use okshell_screenshot::record::RecordHandle;
use crate::menus::menu_widgets::screen_record::recording_service::{recording_state, RecordingStateStoreFields};

#[derive(Debug, Clone)]
pub(crate) struct RecordingIndicatorModel {
    orientation: Orientation,
    recording_handle: Option<RecordHandle>,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum RecordingIndicatorInput {
    StopClicked,
    RecordingHandleChanged(Option<RecordHandle>),
}

#[derive(Debug)]
pub(crate) enum RecordingIndicatorOutput {}

pub(crate) struct RecordingIndicatorInit {
    pub(crate) orientation: Orientation,
}

#[derive(Debug)]
pub(crate) enum RecordingIndicatorCommandOutput {}

#[relm4::component(pub)]
impl Component for RecordingIndicatorModel {
    type CommandOutput = RecordingIndicatorCommandOutput;
    type Input = RecordingIndicatorInput;
    type Output = RecordingIndicatorOutput;
    type Init = RecordingIndicatorInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "recording-indicator-bar-widget",
            set_hexpand: model.orientation == Orientation::Vertical,
            set_vexpand: model.orientation == Orientation::Horizontal,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            #[watch]
            set_visible: model.recording_handle.is_some(),

            gtk::Button {
                set_css_classes: &["ok-button-error", "ok-bar-widget"],
                set_hexpand: false,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(RecordingIndicatorInput::StopClicked);
                },

                #[name="image"]
                gtk::Image {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_icon_name: Some("record-symbolic"),
                }
            }
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let mut effects = EffectScope::new();

        let recording_state = recording_state();

        let sender_clone = sender.clone();
        effects.push(move |_| {
            let recording_state = recording_state.clone();
            let handle = recording_state.handle().get();
            sender_clone.input(RecordingIndicatorInput::RecordingHandleChanged(handle));
        });

        let recording_handle = recording_state.clone().handle().get_untracked();

        let model = RecordingIndicatorModel {
            orientation: params.orientation,
            recording_handle,
            _effects: effects,
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
            RecordingIndicatorInput::StopClicked => {
                if let Some(handle) = &self.recording_handle {
                    handle.stop();
                }
            }
            RecordingIndicatorInput::RecordingHandleChanged(handle) => {
                self.recording_handle = handle;
            }
        }

        self.update_view(widgets, sender);
    }
}