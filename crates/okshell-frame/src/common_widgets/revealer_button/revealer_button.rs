use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use relm4::gtk::Orientation;
use relm4::gtk::prelude::*;
use std::fmt::Debug;

pub(crate) struct RevealerButtonModel<ContentComponent: Component, RevealedContentComponent: Component> {
    pub content: Controller<ContentComponent>,
    pub revealed_content: Controller<RevealedContentComponent>,
    revealed: bool,
}

#[derive(Debug)]
pub(crate) enum RevealerButtonInput {
    RevealClicked,
    SetRevealed(bool),
}

#[derive(Debug)]
pub(crate) enum RevealerButtonOutput {}

pub(crate) struct RevealerButtonInit<ContentComponent: Component, RevealedContentComponent: Component> {
    pub content: Controller<ContentComponent>,
    pub revealed_content: Controller<RevealedContentComponent>,
}

#[derive(Debug)]
pub(crate) enum RevealerButtonCommandOutput {}

#[relm4::component(pub)]
impl<ContentComponent, RevealedContentComponent> Component for RevealerButtonModel<ContentComponent, RevealedContentComponent>
where
    ContentComponent: Component<Output = ()> + 'static,
    ContentComponent::Input: Debug,
    ContentComponent::Root: IsA<gtk::Widget>,
    RevealedContentComponent: Component + 'static,
    RevealedContentComponent::Output: Debug,
    RevealedContentComponent::Root: IsA<gtk::Widget>,
{
    type CommandOutput = RevealerButtonCommandOutput;
    type Input = RevealerButtonInput;
    type Output = RevealerButtonOutput;
    type Init = RevealerButtonInit<ContentComponent, RevealedContentComponent>;

    view! {
        #[root]
        gtk::Box {
            set_orientation: Orientation::Vertical,

            gtk::Button {
                add_css_class: "ok-button-surface",
                set_hexpand: true,
                set_vexpand: false,
                connect_clicked[sender] => move |_| {
                    sender.input(RevealerButtonInput::RevealClicked);
                },

                model.content.widget().clone() {},
            },

            #[name = "revealer"]
            gtk::Revealer {
                set_margin_top: 10,
                set_transition_duration: 200,
                set_transition_type: gtk::RevealerTransitionType::SlideDown,
                #[watch]
                set_reveal_child: model.revealed,

                #[name = "revealer_button_content"]
                gtk::Box {
                    add_css_class: "revealer-button-content",
                    set_orientation: Orientation::Vertical,
                    
                    model.revealed_content.widget().clone() {},
                },
            },
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = RevealerButtonModel {
            content: params.content,
            revealed_content: params.revealed_content,
            revealed: false,
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
            RevealerButtonInput::RevealClicked => {
                self.revealed = !self.revealed;
                if self.revealed {
                    widgets.revealer_button_content.add_css_class("revealed");
                } else {
                    widgets.revealer_button_content.remove_css_class("revealed");
                }
            }
            RevealerButtonInput::SetRevealed(val) => {
                self.revealed = val;
                if self.revealed {
                    widgets.revealer_button_content.add_css_class("revealed");
                } else {
                    widgets.revealer_button_content.remove_css_class("revealed");
                }
            }
        }

        self.update_view(widgets, sender);
    }
}