use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use relm4::gtk::prelude::WidgetExt;
use crate::common_widgets::revealer_row::revealer_row::{RevealerRowInit, RevealerRowInput, RevealerRowModel, RevealerRowOutput};
use crate::common_widgets::revealer_row::revealer_row_label::{RevealerRowLabelInit, RevealerRowLabelModel};
use crate::menus::menu_widgets::wallpaper::wallpaper_menu_widget::{WallpaperMenuWidgetInit, WallpaperMenuWidgetModel};

pub(crate) struct WallpaperDropdownMenuWidgetModel {
    revealer_row: Controller<RevealerRowModel<RevealerRowLabelModel, WallpaperMenuWidgetModel>>,
}

#[derive(Debug)]
pub(crate) enum WallpaperDropdownMenuWidgetInput {
    RevealerRowRevealed,
    RevealerRowHidden,
    ActionButtonClicked,
    ParentRevealChanged(bool),
}

#[derive(Debug)]
pub(crate) enum WallpaperDropdownMenuWidgetOutput {}

pub(crate) struct WallpaperDropdownMenuWidgetInit {}

#[derive(Debug)]
pub(crate) enum WallpaperDropdownMenuWidgetCommandOutput {}

#[relm4::component(pub)]
impl Component for WallpaperDropdownMenuWidgetModel {
    type CommandOutput = WallpaperDropdownMenuWidgetCommandOutput;
    type Input = WallpaperDropdownMenuWidgetInput;
    type Output = WallpaperDropdownMenuWidgetOutput;
    type Init = WallpaperDropdownMenuWidgetInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "wallpaper-dropdown-menu-widget",
            
            model.revealer_row.widget().clone() {}
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let row_content = RevealerRowLabelModel::builder()
            .launch(RevealerRowLabelInit {
                label: "Wallpaper".to_string(),
            })
            .detach();

        let revealed_content = WallpaperMenuWidgetModel::builder()
            .launch(WallpaperMenuWidgetInit {
                thumbnail_width: 150,
                thumbnail_height: 100,
                row_count: 2,
            })
            .detach();

        let revealer_row = RevealerRowModel::<RevealerRowLabelModel, WallpaperMenuWidgetModel>::builder()
            .launch(RevealerRowInit {
                icon_name: "wallpaper-symbolic".into(),
                action_button_sensitive: false,
                content: row_content,
                revealed_content,
            })
            .forward(sender.input_sender(), |msg| {
                match msg {
                    RevealerRowOutput::ActionButtonClicked => {
                        WallpaperDropdownMenuWidgetInput::ActionButtonClicked
                    }
                    RevealerRowOutput::Revealed => {
                        WallpaperDropdownMenuWidgetInput::RevealerRowRevealed
                    }
                    RevealerRowOutput::Hidden => {
                        WallpaperDropdownMenuWidgetInput::RevealerRowHidden
                    }
                }
            });

        let model = WallpaperDropdownMenuWidgetModel {
            revealer_row,
        };

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
        match message {
            WallpaperDropdownMenuWidgetInput::RevealerRowRevealed => {

            }
            WallpaperDropdownMenuWidgetInput::RevealerRowHidden => {

            }
            WallpaperDropdownMenuWidgetInput::ActionButtonClicked => {

            }
            WallpaperDropdownMenuWidgetInput::ParentRevealChanged(revealed) => {
                if !revealed {
                    self.revealer_row.emit(RevealerRowInput::SetRevealed(false))
                }
            }
        }
    }
}