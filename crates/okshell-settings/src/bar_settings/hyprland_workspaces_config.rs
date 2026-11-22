use std::collections::HashSet;
use std::ffi::c_int;

use okshell_config::schema::bar_widgets::HyprlandWorkspacesConfig;
use relm4::gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, gtk};

/// First dropdown entry; maps to an empty `font`, which shows icons.
const NONE_LABEL: &str = "(none)";

#[derive(Debug)]
pub struct HyprlandWorkspacesConfigModel {
    config: HyprlandWorkspacesConfig,
    fonts: gtk::StringList,
}

#[derive(Debug)]
pub enum HyprlandWorkspacesConfigInput {
    FontSelected(Option<String>),
}

#[derive(Debug)]
pub enum HyprlandWorkspacesConfigOutput {
    Changed(HyprlandWorkspacesConfig),
}

#[relm4::component(pub)]
impl Component for HyprlandWorkspacesConfigModel {
    type CommandOutput = ();
    type Input = HyprlandWorkspacesConfigInput;
    type Output = HyprlandWorkspacesConfigOutput;
    type Init = HyprlandWorkspacesConfig;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 4,
            add_css_class: "hyprland-workspaces-config",

            // Font
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,

                    gtk::Label {
                        add_css_class: "label-small",
                        set_label: "Number font",
                        set_halign: gtk::Align::Start,
                    },

                    gtk::Label {
                        add_css_class: "label-small",
                        set_label: "Show workspace numbers in this font. (none) shows icons.",
                        set_halign: gtk::Align::Start,
                        set_xalign: 0.0,
                        set_wrap: true,
                        set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                    },
                },

                gtk::DropDown {
                    set_width_request: 200,
                    set_valign: gtk::Align::Center,
                    set_model: Some(&model.fonts),
                    set_enable_search: true,
                    set_expression: Some(&gtk::PropertyExpression::new(
                        gtk::StringObject::static_type(),
                        None::<gtk::Expression>,
                        "string",
                    )),
                    // Set before connecting so init doesn't emit a change.
                    set_selected: model.selected_index(),
                    connect_selected_notify[sender] => move |dd| {
                        let selected = dd.selected_item()
                            .and_downcast::<gtk::StringObject>()
                            .map(|s| s.string().to_string());
                        sender.input(HyprlandWorkspacesConfigInput::FontSelected(selected));
                    },
                },
            },
        }
    }

    fn init(
        config: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let available = available_fonts();
        let mut entries: Vec<&str> = Vec::with_capacity(available.len() + 2);
        entries.push(NONE_LABEL);

        // Keep a hand-written value (e.g. "JetBrains Mono Bold 11") selectable
        // instead of silently showing "(none)".
        let current = config.font.trim();
        if !current.is_empty() && !available.iter().any(|f| f == current) {
            entries.push(current);
        }
        entries.extend(available.iter().map(String::as_str));

        let fonts = gtk::StringList::new(&entries);
        let model = HyprlandWorkspacesConfigModel { config, fonts };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            HyprlandWorkspacesConfigInput::FontSelected(font) => {
                self.config.font = match font.as_deref() {
                    Some(NONE_LABEL) | None => String::new(),
                    Some(font) => font.to_string(),
                };
            }
        }
        let _ = sender.output(HyprlandWorkspacesConfigOutput::Changed(self.config.clone()));
    }
}

impl HyprlandWorkspacesConfigModel {
    fn selected_index(&self) -> u32 {
        let current = self.config.font.trim();
        if current.is_empty() {
            return 0;
        }
        (0..self.fonts.n_items())
            .find(|&i| self.fonts.string(i).as_deref() == Some(current))
            .unwrap_or(0)
    }
}

unsafe extern "C" {
    /// From libfontconfig, which the `fontconfig` crate already links.
    fn FcInitReinitialize() -> c_int;
}

fn available_fonts() -> Vec<String> {
    // fontconfig caches its font list for the whole process; force a reload.
    unsafe {
        FcInitReinitialize();
    }

    let Some(fc) = fontconfig::Fontconfig::new() else {
        return vec![];
    };
    let Ok(pattern) = fontconfig::Pattern::new(&fc) else {
        return vec![];
    };
    let Ok(font_set) = fontconfig::list_fonts(&pattern, None) else {
        return vec![];
    };

    let mut families = HashSet::new();
    for font_pattern in font_set.iter() {
        if let Ok(family) = font_pattern.get_string(fontconfig::FC_FAMILY) {
            families.insert(family.to_string());
        }
    }

    let mut families: Vec<_> = families.into_iter().collect();
    families.sort();
    families
}
