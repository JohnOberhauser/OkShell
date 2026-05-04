use relm4::gtk::Orientation;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentController, ComponentParts, ComponentSender, Controller, gtk};
use std::fmt::Debug;

pub(crate) struct RevealerRowModel<ContentComponent: Component, RevealedContentComponent: Component>
{
    pub content: Controller<ContentComponent>,
    pub revealed_content: Controller<RevealedContentComponent>,
    revealed: bool,
    action_button_sensitive: bool,
}

#[derive(Debug)]
pub(crate) enum RevealerRowInput {
    UpdateActionIconName(String),
    RevealClicked,
    SetRevealed(bool),
}

#[derive(Debug)]
pub(crate) enum RevealerRowOutput {
    ActionButtonClicked,
    Revealed,
    Hidden,
}

pub(crate) struct RevealerRowInit<ContentComponent: Component, RevealedContentComponent: Component>
{
    pub icon_name: String,
    pub action_button_sensitive: bool,
    pub content: Controller<ContentComponent>,
    pub revealed_content: Controller<RevealedContentComponent>,
}

#[derive(Debug)]
pub(crate) enum RevealerRowCommandOutput {}

#[relm4::component(pub)]
impl<ContentComponent, RevealedContentComponent> Component
    for RevealerRowModel<ContentComponent, RevealedContentComponent>
where
    ContentComponent: Component + 'static,
    ContentComponent::Output: Debug,
    ContentComponent::Input: Debug,
    ContentComponent::Root: IsA<gtk::Widget>,
    RevealedContentComponent: Component + 'static,
    RevealedContentComponent::Output: Debug,
    RevealedContentComponent::Root: IsA<gtk::Widget>,
{
    type CommandOutput = RevealerRowCommandOutput;
    type Input = RevealerRowInput;
    type Output = RevealerRowOutput;
    type Init = RevealerRowInit<ContentComponent, RevealedContentComponent>;

    view! {
        #[root]
        gtk::Box {
            set_orientation: Orientation::Vertical,

            gtk::Box {
                set_orientation: Orientation::Horizontal,

                gtk::Button {
                    set_css_classes: &["ok-button-surface", "ok-button-medium", "ok-button-no-disabled"],
                    set_hexpand: false,
                    set_vexpand: false,
                    set_margin_end: 10,
                    #[watch]
                    set_sensitive: model.action_button_sensitive,
                    connect_clicked[sender] => move |_| {
                        sender.output(RevealerRowOutput::ActionButtonClicked).unwrap_or_default();
                    },

                    #[name = "action_icon_image"]
                    gtk::Image {
                        add_css_class: "revealer-row-action-button-inner",
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                    },
                },

                model.content.widget().clone() {},

                gtk::Button {
                    set_css_classes: &["ok-button-surface", "ok-button-medium-thin"],
                    set_hexpand: false,
                    set_vexpand: false,
                    connect_clicked[sender] => move |_| {
                        sender.input(RevealerRowInput::RevealClicked);
                    },

                    #[name = "reveal_icon_image"]
                    gtk::Image {
                        #[watch]
                        set_css_classes: if model.revealed {
                            &["revealer-row-action-button-inner", "revealed"]
                        } else {
                            &["revealer-row-action-button-inner"]
                        },
                        set_hexpand: true,
                        set_vexpand: true,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        set_icon_name: Some("menu-right-symbolic")
                    },
                },
            },

            #[name = "revealer"]
            gtk::Revealer {
                set_margin_top: 10,
                set_transition_duration: 200,
                set_transition_type: gtk::RevealerTransitionType::SlideDown,
                #[watch]
                set_reveal_child: model.revealed,

                #[name = "revealer_row_content"]
                gtk::Box {
                    #[watch]
                    set_css_classes: if model.revealed {
                            &["revealer-row-content", "revealed"]
                        } else {
                            &["revealer-row-content"]
                        },
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
        let model = RevealerRowModel {
            content: params.content,
            revealed_content: params.revealed_content,
            revealed: false,
            action_button_sensitive: params.action_button_sensitive,
        };

        let widgets = view_output!();

        widgets
            .action_icon_image
            .set_icon_name(Some(&params.icon_name));

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
            RevealerRowInput::UpdateActionIconName(name) => {
                widgets.action_icon_image.set_icon_name(Some(&name));
            }
            RevealerRowInput::RevealClicked => {
                self.revealed = !self.revealed;
                if self.revealed {
                    let _ = sender.output(RevealerRowOutput::Revealed);
                } else {
                    let _ = sender.output(RevealerRowOutput::Hidden);
                }
            }
            RevealerRowInput::SetRevealed(val) => {
                self.revealed = val;
                if self.revealed {
                    let _ = sender.output(RevealerRowOutput::Revealed);
                } else {
                    let _ = sender.output(RevealerRowOutput::Hidden);
                }
            }
        }

        self.update_view(widgets, sender);
    }
}
