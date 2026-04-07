use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::factory::{DynamicIndex, FactoryVecDeque};
use relm4::gtk::gio;
use relm4::gtk::prelude::{ActionMapExt, BoxExt, OrientableExt, WidgetExt};
use okshell_config::schema::menu_widgets::MenuWidget;
use crate::menu_settings::menu_widget_row::{MenuWidgetRowModel, MenuWidgetRowOutput};

pub struct MenuWidgetListInit {
    pub widgets: Vec<MenuWidget>,
    pub draw_border: bool,
}

#[derive(Debug)]
pub struct MenuWidgetListModel {
    widgets: FactoryVecDeque<MenuWidgetRowModel>,
}

#[derive(Debug)]
pub enum MenuWidgetListInput {
    AddWidget(MenuWidget),
    RemoveWidget(DynamicIndex),
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
    WidgetChanged(DynamicIndex, MenuWidget),
    SetWidgetsEffect(Vec<MenuWidget>),
}

#[derive(Debug)]
pub enum MenuWidgetListOutput {
    Changed(Vec<MenuWidget>),
}

#[relm4::component(pub)]
impl Component for MenuWidgetListModel {
    type CommandOutput = ();
    type Input = MenuWidgetListInput;
    type Output = MenuWidgetListOutput;
    type Init = MenuWidgetListInit;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 4,
            add_css_class: if init.draw_border {
                "settings-menu-widget-section"
            } else {
                ""
            },

            #[local_ref]
            widget_list -> gtk::ListBox {
                add_css_class: "settings-menu-widget-section-list",
                set_selection_mode: gtk::SelectionMode::None,
            },

            #[name = "add_button"]
            gtk::MenuButton {
                set_label: "Add widget",
                set_halign: gtk::Align::Start,
                set_always_show_arrow: false,
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut widgets = FactoryVecDeque::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |output| match output {
                MenuWidgetRowOutput::MoveUp(idx) => MenuWidgetListInput::MoveUp(idx),
                MenuWidgetRowOutput::MoveDown(idx) => MenuWidgetListInput::MoveDown(idx),
                MenuWidgetRowOutput::Remove(idx) => MenuWidgetListInput::RemoveWidget(idx),
                MenuWidgetRowOutput::WidgetChanged(idx, w) => {
                    MenuWidgetListInput::WidgetChanged(idx, w)
                }
            });

        {
            let mut guard = widgets.guard();
            for w in init.widgets {
                guard.push_back(w);
            }
        }

        let model = MenuWidgetListModel { widgets };

        let widget_list = model.widgets.widget();
        let view_widgets = view_output!();

        Self::build_add_menu(&view_widgets.add_button, &sender);

        ComponentParts { model, widgets: view_widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            MenuWidgetListInput::AddWidget(widget) => {
                self.widgets.guard().push_back(widget);
                self.emit_changed(&sender);
            }
            MenuWidgetListInput::RemoveWidget(index) => {
                self.widgets.guard().remove(index.current_index());
                self.emit_changed(&sender);
            }
            MenuWidgetListInput::MoveUp(index) => {
                let idx = index.current_index();
                if idx > 0 {
                    self.widgets.guard().move_to(idx, idx - 1);
                    self.emit_changed(&sender);
                }
            }
            MenuWidgetListInput::MoveDown(index) => {
                let idx = index.current_index();
                if idx + 1 < self.widgets.len() {
                    self.widgets.guard().move_to(idx, idx + 1);
                    self.emit_changed(&sender);
                }
            }
            MenuWidgetListInput::WidgetChanged(index, widget) => {
                let idx = index.current_index();
                if let Some(row) = self.widgets.guard().get_mut(idx) {
                    row.widget = widget;
                }
                self.emit_changed(&sender);
            }
            MenuWidgetListInput::SetWidgetsEffect(new_widgets) => {
                let mut guard = self.widgets.guard();
                guard.clear();
                for widget in new_widgets {
                    guard.push_back(widget);
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

impl MenuWidgetListModel {
    fn emit_changed(&self, sender: &ComponentSender<Self>) {
        let widgets: Vec<MenuWidget> = self.widgets.iter()
            .map(|row| row.widget.clone())
            .collect();
        let _ = sender.output(MenuWidgetListOutput::Changed(widgets));
    }

    fn build_add_menu(button: &gtk::MenuButton, sender: &ComponentSender<Self>) {
        let menu = gio::Menu::new();
        let action_group = gio::SimpleActionGroup::new();

        for widget in MenuWidget::all_defaults() {
            let action_name = widget.action_name();
            let action = gio::SimpleAction::new(&action_name, None);

            let sender = sender.input_sender().clone();
            let widget_clone = widget.clone();
            action.connect_activate(move |_, _| {
                sender.emit(MenuWidgetListInput::AddWidget(widget_clone.clone()));
            });

            action_group.add_action(&action);
            menu.append(
                Some(widget.display_name()),
                Some(&format!("menuwidget.{action_name}")),
            );
        }

        button.insert_action_group("menuwidget", Some(&action_group));
        button.set_menu_model(Some(&menu));
    }
}