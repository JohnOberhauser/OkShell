use std::time::Duration;
use notify_rust::Notification;
use relm4::gtk::prelude::*;
use relm4::{gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller};
use okshell_screenshot::{take_screenshot, CaptureArea, OutputTarget, ScreenshotRequest, ScreenshotResult};
use okshell_sounds::play_shutter;
use crate::common_widgets::big_button::BigButton;
use crate::common_widgets::option_list::{OptionsListOutput};
use crate::common_widgets::revealer_row::revealer_row::{RevealerRowInit, RevealerRowInput, RevealerRowModel};
use crate::common_widgets::revealer_row::revealer_row_label::{RevealerRowLabelInit, RevealerRowLabelInput, RevealerRowLabelModel};
use crate::menus::menu_widgets::screenshot::delay_option::{DelayOption, DelayOptionsList};
use crate::menus::menu_widgets::screenshot::save_option::{SaveOptionRow, SaveOptionsList};
use okshell_utils::notifications::show_file_saved_notification;

pub(crate) struct ScreenshotMenuWidgetModel {
    delay: u32,
    delay_row: Controller<RevealerRowModel<RevealerRowLabelModel, DelayOptionsList>>,
    save_option: OutputTarget,
    save_row: Controller<RevealerRowModel<RevealerRowLabelModel, SaveOptionsList>>,
}

#[derive(Debug)]
pub(crate) enum ScreenshotMenuWidgetInput {
    DelaySelected(DelayOption),
    SaveOptionSelected(SaveOptionRow),
    AllClicked,
    MonitorClicked,
    WindowClicked,
    AreaClicked,
}

#[derive(Debug)]
pub(crate) enum ScreenshotMenuWidgetOutput {
    CloseMenu,
}

pub(crate) struct ScreenshotMenuWidgetInit {}

#[derive(Debug)]
pub(crate) enum ScreenshotMenuWidgetCommandOutput {}

#[relm4::component(pub)]
impl Component for ScreenshotMenuWidgetModel {
    type CommandOutput = ScreenshotMenuWidgetCommandOutput;
    type Input = ScreenshotMenuWidgetInput;
    type Output = ScreenshotMenuWidgetOutput;
    type Init = ScreenshotMenuWidgetInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "screenshot-menu-widget",
            set_hexpand: false,
            set_orientation: gtk::Orientation::Vertical,

            gtk::Label {
                add_css_class: "label-large-bold",
                set_label: "Screenshot",
                set_margin_bottom: 8,
            },

            model.delay_row.widget().clone() {},
            model.save_row.widget().clone() {},

            gtk::Box {
                set_margin_top: 8,
                set_orientation: gtk::Orientation::Vertical,
                set_halign: gtk::Align::Center,
                set_spacing: 16,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 32,

                    #[template]
                    BigButton {
                        connect_clicked[sender] => move |_| {
                            sender.input(ScreenshotMenuWidgetInput::AllClicked);
                        },

                        #[template_child]
                        icon {
                            set_icon_name: Some("screenshot-all-symbolic"),
                        },
                        #[template_child]
                        label {
                            set_label: "All",
                        },
                    },

                    #[template]
                    BigButton {
                        connect_clicked[sender] => move |_| {
                            sender.input(ScreenshotMenuWidgetInput::MonitorClicked);
                        },

                        #[template_child]
                        icon {
                            set_icon_name: Some("screenshot-monitor-symbolic"),
                        },
                        #[template_child]
                        label {
                            set_label: "Monitor",
                        },
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 32,

                    #[template]
                    BigButton {
                        connect_clicked[sender] => move |_| {
                            sender.input(ScreenshotMenuWidgetInput::WindowClicked);
                        },

                        #[template_child]
                        icon {
                            set_icon_name: Some("screenshot-window-symbolic"),
                        },
                        #[template_child]
                        label {
                            set_label: "Window",
                        },
                    },

                    #[template]
                    BigButton {
                        connect_clicked[sender] => move |_| {
                            sender.input(ScreenshotMenuWidgetInput::AreaClicked);
                        },

                        #[template_child]
                        icon {
                            set_icon_name: Some("screenshot-area-symbolic"),
                        },
                        #[template_child]
                        label {
                            set_label: "Area",
                        },
                    },
                },
            },
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {

        let delay_row_content = RevealerRowLabelModel::builder()
            .launch(RevealerRowLabelInit {
                label: "Delay: 0 seconds".to_string(),
            })
            .detach();

        let delay_row_revealed_content = DelayOptionsList::builder()
            .launch(vec![
                DelayOption { value: 0, icon_name: "timer-symbolic".into() },
                DelayOption { value: 1, icon_name: "timer-symbolic".into() },
                DelayOption { value: 3, icon_name: "timer-symbolic".into() },
                DelayOption { value: 5, icon_name: "timer-symbolic".into() },
                DelayOption { value: 10, icon_name: "timer-symbolic".into() },
            ])
            .forward(sender.input_sender(), |msg| {
                match msg {
                    OptionsListOutput::Selected(opt) => {
                        ScreenshotMenuWidgetInput::DelaySelected(opt)
                    } 
                }
            });
        
        let delay_row = RevealerRowModel::<RevealerRowLabelModel, DelayOptionsList>::builder()
            .launch(RevealerRowInit {
                icon_name: "timer-symbolic".into(),
                action_button_sensitive: false,
                content: delay_row_content,
                revealed_content: delay_row_revealed_content,
            })
            .detach();

        let save_option_row_content = RevealerRowLabelModel::builder()
            .launch(RevealerRowLabelInit {
                label: "Save to file and clipboard".to_string(),
            })
            .detach();

        let save_option_row_revealed_content = SaveOptionsList::builder()
            .launch(vec![
                SaveOptionRow { value: OutputTarget::FileAndClipboard, icon_name: "screenshot-save-both-symbolic".into() },
                SaveOptionRow { value: OutputTarget::Clipboard, icon_name: "screenshot-save-clipboard-symbolic".into() },
                SaveOptionRow { value: OutputTarget::File, icon_name: "screenshot-save-file-symbolic".into() },
            ])
            .forward(sender.input_sender(), |msg| {
                match msg {
                    OptionsListOutput::Selected(opt) => {
                        ScreenshotMenuWidgetInput::SaveOptionSelected(opt)
                    }
                }
            });

        let save_option_row = RevealerRowModel::<RevealerRowLabelModel, SaveOptionsList>::builder()
            .launch(RevealerRowInit {
                icon_name: "screenshot-save-both-symbolic".into(),
                action_button_sensitive: false,
                content: save_option_row_content,
                revealed_content: save_option_row_revealed_content,
            })
            .detach();

        let model = ScreenshotMenuWidgetModel {
            delay: 0,
            delay_row,
            save_option: OutputTarget::FileAndClipboard,
            save_row: save_option_row,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ScreenshotMenuWidgetInput::DelaySelected(opt) => {
                self.delay = opt.value;
                self.delay_row.model().content.emit(RevealerRowLabelInput::SetLabel(
                    if opt.value == 1 {
                        format!("Delay: {} second", opt.value)
                    } else {
                        format!("Delay: {} seconds", opt.value)
                    }
                ));
                self.delay_row.emit(RevealerRowInput::SetRevealed(false));
            }
            ScreenshotMenuWidgetInput::SaveOptionSelected(opt) => {
                self.save_option = opt.value.clone();
                self.save_row.model().content.emit(RevealerRowLabelInput::SetLabel(
                    match opt.value {
                        OutputTarget::FileAndClipboard => {
                            "Save to file and clipboard".to_string()
                        }
                        OutputTarget::File => {
                            "Save to file".to_string()
                        }
                        OutputTarget::Clipboard => {
                            "Save to clipboard".to_string()
                        }
                    },
                ));
                self.save_row.emit(RevealerRowInput::UpdateActionIconName(opt.icon_name));
                self.save_row.emit(RevealerRowInput::SetRevealed(false));
            }
            ScreenshotMenuWidgetInput::AllClicked => {
                let _ = sender.output(ScreenshotMenuWidgetOutput::CloseMenu);
                take_screenshot(
                    ScreenshotRequest {
                        area: CaptureArea::All,
                        target: self.save_option.clone(),
                    },
                    Duration::from_secs(self.delay as u64).max(Duration::from_millis(500)),
                    |result| {
                        match result {
                            Ok(r) => complete_screenshot(r),
                            Err(e) => eprintln!("screenshot failed: {e}"),
                        }
                    },
                );
            }
            ScreenshotMenuWidgetInput::MonitorClicked => {
                let _ = sender.output(ScreenshotMenuWidgetOutput::CloseMenu);
                take_screenshot(
                    ScreenshotRequest {
                        area: CaptureArea::SelectMonitor,
                        target: self.save_option.clone(),
                    },
                    Duration::from_secs(self.delay as u64),
                    move |result| {
                        match result {
                            Ok(r) => complete_screenshot(r),
                            Err(e) => eprintln!("screenshot failed: {e}"),
                        }
                    },
                );
            }
            ScreenshotMenuWidgetInput::WindowClicked => {
                let _ = sender.output(ScreenshotMenuWidgetOutput::CloseMenu);
                take_screenshot(
                    ScreenshotRequest {
                        area: CaptureArea::SelectWindow,
                        target: self.save_option.clone(),
                    },
                    Duration::from_secs(self.delay as u64),
                    move |result| {
                        match result {
                            Ok(r) => complete_screenshot(r),
                            Err(e) => eprintln!("screenshot failed: {e}"),
                        }
                    },
                );
            }
            ScreenshotMenuWidgetInput::AreaClicked => {
                let _ = sender.output(ScreenshotMenuWidgetOutput::CloseMenu);
                take_screenshot(
                    ScreenshotRequest {
                        area: CaptureArea::SelectRegion,
                        target: self.save_option.clone(),
                    },
                    Duration::from_secs(self.delay as u64),
                    |result| {
                        match result {
                            Ok(r) => complete_screenshot(r),
                            Err(e) => eprintln!("screenshot failed: {e}"),
                        }
                    },
                );
            }
        }
    }
}

fn complete_screenshot(result: ScreenshotResult) {
    play_shutter();

    match (result.saved_path, result.in_clipboard) {
        (Some(path), true) => {
            show_file_saved_notification("Screenshot saved & copied".to_string(), path);
        }
        (Some(path), false) => {
            show_file_saved_notification("Screenshot saved".to_string(), path);
        }
        (None, true) => {
            Notification::new()
                .summary("Screenshot copied to clipboard")
                .appname("okshell")
                .show()
                .ok();
        }
        (None, false) => {}
    }
}
