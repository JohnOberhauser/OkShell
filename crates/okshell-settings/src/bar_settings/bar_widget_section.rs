use relm4::{gtk, Component, ComponentParts, ComponentSender};
use relm4::factory::{DynamicIndex, FactoryVecDeque};
use relm4::gtk::gio;
use relm4::gtk::prelude::{ActionMapExt, BoxExt, OrientableExt, WidgetExt};
use okshell_config::schema::bar_widgets::BarWidget;
use crate::bar_settings::bar_widget_factory::{ActiveWidgetModel, ActiveWidgetOutput};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BarSection {
    Start,
    Center,
    End,
}

impl BarSection {
    pub fn display_name(&self) -> &'static str {
        match self {
            BarSection::Start => "Start",
            BarSection::Center => "Center",
            BarSection::End => "End",
        }
    }
}

#[derive(Debug)]
pub struct WidgetSectionModel {
    section: BarSection,
    widgets: FactoryVecDeque<ActiveWidgetModel>,
}

#[derive(Debug)]
pub enum WidgetSectionInput {
    AddWidget(BarWidget),
    RemoveWidget(DynamicIndex),
    MoveUp(DynamicIndex),
    MoveDown(DynamicIndex),
}

#[derive(Debug)]
pub enum WidgetSectionOutput {
    Changed(Vec<BarWidget>),
}

pub struct WidgetSectionInit {
    pub bar_section: BarSection,
    pub widgets: Vec<BarWidget>,
}

#[relm4::component(pub)]
impl Component for WidgetSectionModel {
    type CommandOutput = ();
    type Input = WidgetSectionInput;
    type Output = WidgetSectionOutput;
    type Init = WidgetSectionInit;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,
            add_css_class: "settings-bar-widget-section",

            gtk::Label {
                add_css_class: "label-medium-bold",
                set_halign: gtk::Align::Start,
                #[watch]
                set_label: model.section.display_name(),
            },

            #[local_ref]
            widget_list -> gtk::ListBox {
                set_selection_mode: gtk::SelectionMode::None,
                add_css_class: "settings-bar-widget-section-list",
            },

            #[name = "add_widget_button"]
            gtk::MenuButton {
                add_css_class: "ok-button-primary",
                set_label: "Add widget",
                set_halign: gtk::Align::Start,
                set_always_show_arrow: false,
            },
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut widgets = FactoryVecDeque::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |output| match output {
                ActiveWidgetOutput::MoveUp(idx) => WidgetSectionInput::MoveUp(idx),
                ActiveWidgetOutput::MoveDown(idx) => WidgetSectionInput::MoveDown(idx),
                ActiveWidgetOutput::Remove(idx) => WidgetSectionInput::RemoveWidget(idx),
            });

        params.widgets.iter().for_each(|widget| {
            widgets.guard().push_back(widget.clone());
        });

        let model = WidgetSectionModel {
            section: params.bar_section,
            widgets,
        };

        let widget_list = model.widgets.widget();
        let widgets_view = view_output!();

        // Build the add-widget menu
        Self::build_add_menu(&widgets_view.add_widget_button, &sender);

        ComponentParts { model, widgets: widgets_view }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            WidgetSectionInput::AddWidget(widget) => {
                self.widgets.guard().push_back(widget);
                self.emit_changed(&sender);
            }
            WidgetSectionInput::RemoveWidget(index) => {
                let idx = index.current_index();
                self.widgets.guard().remove(idx);
                self.emit_changed(&sender);
            }
            WidgetSectionInput::MoveUp(index) => {
                let idx = index.current_index();
                if idx > 0 {
                    self.widgets.guard().move_to(idx, idx - 1);
                    self.emit_changed(&sender);
                }
            }
            WidgetSectionInput::MoveDown(index) => {
                let idx = index.current_index();
                let len = self.widgets.len();
                if idx + 1 < len {
                    self.widgets.guard().move_to(idx, idx + 1);
                    self.emit_changed(&sender);
                }
            }
        }

        self.update_view(widgets, sender);
    }
}

impl WidgetSectionModel {
    fn emit_changed(&self, sender: &ComponentSender<Self>) {
        let widgets: Vec<BarWidget> = self.widgets.iter()
            .map(|w| w.widget.clone())
            .collect();
        let _ = sender.output(WidgetSectionOutput::Changed(
            widgets,
        ));
    }

    fn build_add_menu(button: &gtk::MenuButton, sender: &ComponentSender<Self>) {
        let menu = gio::Menu::new();
        let action_group = gio::SimpleActionGroup::new();

        for widget in BarWidget::all() {
            let action_name = widget.action_name();
            let action = gio::SimpleAction::new(&action_name, None);

            let sender = sender.input_sender().clone();
            action.connect_activate(move |_, _| {
                sender.emit(WidgetSectionInput::AddWidget(widget.clone()));
            });

            action_group.add_action(&action);
            menu.append(
                Some(widget.display_name()),
                Some(&format!("widget.{action_name}")),
            );
        }

        button.insert_action_group("widget", Some(&action_group));
        button.set_menu_model(Some(&menu));
    }
}