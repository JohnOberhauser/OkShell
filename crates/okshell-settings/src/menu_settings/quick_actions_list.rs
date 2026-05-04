use crate::menu_settings::quick_actions_row::{QuickActionRowModel, QuickActionRowOutput};
use okshell_config::schema::menu_widgets::QuickActionWidget;
use relm4::factory::{DynamicIndex, FactoryVecDeque};
use relm4::gtk::gio;
use relm4::gtk::prelude::{ActionMapExt, BoxExt, OrientableExt, WidgetExt};
use relm4::{Component, ComponentParts, ComponentSender, gtk};

#[derive(Debug)]
pub struct QuickActionListModel {
    actions: FactoryVecDeque<QuickActionRowModel>,
}

#[derive(Debug)]
pub enum QuickActionListInput {
    AddAction(QuickActionWidget),
    RemoveAction(DynamicIndex),
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
}

#[derive(Debug)]
pub enum QuickActionListOutput {
    Changed(Vec<QuickActionWidget>),
}

#[relm4::component(pub)]
impl Component for QuickActionListModel {
    type CommandOutput = ();
    type Input = QuickActionListInput;
    type Output = QuickActionListOutput;
    type Init = Vec<QuickActionWidget>;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 4,

            #[local_ref]
            action_list -> gtk::ListBox {
                add_css_class: "settings-menu-widget-section-list",
                set_selection_mode: gtk::SelectionMode::None,
            },

            #[name = "add_action_button"]
            gtk::MenuButton {
                set_label: "Add action",
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
        let mut actions = FactoryVecDeque::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |output| match output {
                QuickActionRowOutput::Remove(idx) => QuickActionListInput::RemoveAction(idx),
                QuickActionRowOutput::MoveUp(idx) => QuickActionListInput::MoveUp(idx),
                QuickActionRowOutput::MoveDown(idx) => QuickActionListInput::MoveDown(idx),
            });

        {
            let mut guard = actions.guard();
            for action in init {
                guard.push_back(action);
            }
        }

        let model = QuickActionListModel { actions };

        let action_list = model.actions.widget();
        let widgets = view_output!();

        Self::build_add_menu(&widgets.add_action_button, &sender);

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
            QuickActionListInput::AddAction(action) => {
                self.actions.guard().push_back(action);
                self.emit_changed(&sender);
                self.rebuild_add_menu(widgets, &sender);
            }
            QuickActionListInput::RemoveAction(index) => {
                self.actions.guard().remove(index.current_index());
                self.emit_changed(&sender);
                self.rebuild_add_menu(widgets, &sender);
            }
            QuickActionListInput::MoveUp(index) => {
                let idx = index.current_index();
                if idx > 0 {
                    self.actions.guard().move_to(idx, idx - 1);
                    self.emit_changed(&sender);
                }
            }
            QuickActionListInput::MoveDown(index) => {
                let idx = index.current_index();
                if idx + 1 < self.actions.len() {
                    self.actions.guard().move_to(idx, idx + 1);
                    self.emit_changed(&sender);
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

impl QuickActionListModel {
    fn emit_changed(&self, sender: &ComponentSender<Self>) {
        let actions: Vec<QuickActionWidget> =
            self.actions.iter().map(|row| row.action.clone()).collect();
        let _ = sender.output(QuickActionListOutput::Changed(actions));
    }

    fn selected_actions(&self) -> Vec<&QuickActionWidget> {
        self.actions.iter().map(|row| &row.action).collect()
    }

    fn build_add_menu(button: &gtk::MenuButton, sender: &ComponentSender<Self>) {
        let menu = gio::Menu::new();
        let action_group = gio::SimpleActionGroup::new();

        for action in QuickActionWidget::all() {
            let action_name = action.action_name();
            let gio_action = gio::SimpleAction::new(&action_name, None);

            let sender = sender.input_sender().clone();
            let action_clone = action.clone();
            gio_action.connect_activate(move |_, _| {
                sender.emit(QuickActionListInput::AddAction(action_clone.clone()));
            });

            action_group.add_action(&gio_action);
            menu.append(
                Some(action.display_name()),
                Some(&format!("quickaction.{action_name}")),
            );
        }

        button.insert_action_group("quickaction", Some(&action_group));
        button.set_menu_model(Some(&menu));
    }

    fn rebuild_add_menu(
        &self,
        widgets: &<Self as Component>::Widgets,
        sender: &ComponentSender<Self>,
    ) {
        let selected = self.selected_actions();
        let menu = gio::Menu::new();
        let action_group = gio::SimpleActionGroup::new();

        for action in QuickActionWidget::all() {
            if selected.contains(&&action) {
                continue;
            }

            let action_name = action.action_name();
            let gio_action = gio::SimpleAction::new(&action_name, None);

            let sender = sender.input_sender().clone();
            let action_clone = action.clone();
            gio_action.connect_activate(move |_, _| {
                sender.emit(QuickActionListInput::AddAction(action_clone.clone()));
            });

            action_group.add_action(&gio_action);
            menu.append(
                Some(action.display_name()),
                Some(&format!("quickaction.{action_name}")),
            );
        }

        widgets
            .add_action_button
            .insert_action_group("quickaction", Some(&action_group));
        widgets.add_action_button.set_menu_model(Some(&menu));
    }
}
