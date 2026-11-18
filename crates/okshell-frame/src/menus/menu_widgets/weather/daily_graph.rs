use std::cell::RefCell;
use std::rc::Rc;

use crate::menus::menu_widgets::weather::graph_common::{
    AXIS_LABEL_PAD, GRAPH_HEIGHT, Geometry, GraphTheme, ICON_ROW_HEIGHT, TempScale, append_smooth,
    content_width, draw_axis_column, draw_bottom_labels, draw_horizontal_grid, draw_vertical_grid,
    follow_plot_width, item_spacing, item_x, rebuild_icon_row, set_themed_draw_func, stroke_curve,
    temp_ticks, temp_unit_label, with_edge_leads,
};
use okshell_common::scoped_effects::EffectScope;
use okshell_config::config_manager::config_manager;
use okshell_config::schema::config::{ConfigStoreFields, GeneralStoreFields};
use okshell_utils::scroll_extensions::wire_vertical_to_horizontal;
use okshell_utils::weather::{get_temperature_value, get_weather_icon_name};
use reactive_graph::traits::{Get, GetUntracked};
use relm4::gtk::prelude::*;
use relm4::gtk::{self, cairo};
use relm4::{Component, ComponentParts, ComponentSender};
use wayle_weather::{DailyForecast, TemperatureUnit};

/// Horizontal space per day. Days are few, so this is much wider than hourly.
const PX_PER_DAY: i32 = 48;

/// Fixed (non-scrolling) y-axis: a single temperature column.
const AXIS_WIDTH: i32 = 34;

/// Opacity of the band filled between the high and low lines.
const BAND_ALPHA: f64 = 0.15;

struct Point {
    high: f64,
    low: f64,
    day_label: String,
}

#[derive(Default)]
struct GraphData {
    points: Vec<Point>,
    scale: TempScale,
    /// "°C" / "°F", shown under the temp axis column.
    unit_label: &'static str,
}

pub(crate) struct DailyGraphModel {
    daily: Vec<DailyForecast>,
    temperature_unit: TemperatureUnit,
    /// Shared with both draw funcs. Rebuilt on input, read on every draw.
    data: Rc<RefCell<GraphData>>,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum DailyGraphInput {
    Update(Vec<DailyForecast>),
    UpdateTemperatureUnit(TemperatureUnit),
}

#[derive(Debug)]
pub(crate) enum DailyGraphOutput {}

pub(crate) struct DailyGraphInit {
    pub daily: Vec<DailyForecast>,
}

#[derive(Debug)]
pub(crate) enum DailyGraphCommandOutput {}

#[relm4::component(pub)]
impl Component for DailyGraphModel {
    type CommandOutput = DailyGraphCommandOutput;
    type Input = DailyGraphInput;
    type Output = DailyGraphOutput;
    type Init = DailyGraphInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "weather-container",
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,

            gtk::Label {
                add_css_class: "label-large-bold",
                set_label: "Daily Conditions",
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,

                // Fixed y-axis: lives outside the ScrolledWindow so it never scrolls.
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    // Empty space level with the icon row.
                    gtk::Box {
                        set_height_request: ICON_ROW_HEIGHT,
                    },

                    #[name = "axis"]
                    gtk::DrawingArea {
                        add_css_class: "daily-graph",
                        add_css_class: "daily-graph-axis",
                        set_content_width: AXIS_WIDTH,
                        set_content_height: GRAPH_HEIGHT,
                    },
                },

                #[name = "scroll_window"]
                gtk::ScrolledWindow {
                    set_vscrollbar_policy: gtk::PolicyType::Never,
                    set_hscrollbar_policy: gtk::PolicyType::External,
                    set_propagate_natural_height: true,
                    set_hexpand: true,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        #[name = "icons"]
                        gtk::Fixed {
                            add_css_class: "daily-graph-icons",
                            set_height_request: ICON_ROW_HEIGHT,
                        },

                        #[name = "area"]
                        gtk::DrawingArea {
                            add_css_class: "daily-graph",
                            set_content_height: GRAPH_HEIGHT,
                        }
                    }
                },
            },
        }
    }

    fn init(
        params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let base_config = config_manager().config();
        let mut effects = EffectScope::new();

        let config = base_config.clone();
        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config.clone();
            let unit = config.general().temperature_unit().get();
            sender_clone.input(DailyGraphInput::UpdateTemperatureUnit(
                TemperatureUnit::from(unit),
            ));
        });

        let config = base_config.clone();
        let temperature_unit = config.general().temperature_unit().get_untracked();

        let model = DailyGraphModel {
            daily: params.daily,
            temperature_unit: TemperatureUnit::from(temperature_unit),
            data: Rc::new(RefCell::new(GraphData::default())),
            _effects: effects,
        };

        let widgets = view_output!();

        set_themed_draw_func(&widgets.area, model.data.clone(), draw_plot);
        set_themed_draw_func(&widgets.axis, model.data.clone(), draw_axis);
        follow_plot_width(&widgets.area, &widgets.icons);

        model.rebuild(&widgets.area, &widgets.axis, &widgets.icons);
        wire_vertical_to_horizontal(&widgets.scroll_window, 32.0);

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
            DailyGraphInput::Update(daily) => self.daily = daily,
            DailyGraphInput::UpdateTemperatureUnit(unit) => self.temperature_unit = unit,
        }

        self.rebuild(&widgets.area, &widgets.axis, &widgets.icons);
        self.update_view(widgets, sender);
    }
}

impl DailyGraphModel {
    /// Convert forecasts into plot-ready points, resize, and queue redraws.
    fn rebuild(&self, area: &gtk::DrawingArea, axis: &gtk::DrawingArea, icons: &gtk::Fixed) {
        let unit = &self.temperature_unit;
        let points: Vec<Point> = self
            .daily
            .iter()
            .map(|d| Point {
                high: get_temperature_value(&d.temp_high, unit) as f64,
                low: get_temperature_value(&d.temp_low, unit) as f64,
                day_label: d.date.format("%a").to_string(),
            })
            .collect();

        let scale = TempScale::fit(points.iter().flat_map(|p| [p.high, p.low]));

        *self.data.borrow_mut() = GraphData {
            points,
            scale,
            unit_label: temp_unit_label(unit),
        };

        let width = content_width(self.daily.len(), PX_PER_DAY);
        area.set_content_width(width);
        area.queue_draw();
        axis.queue_draw();

        rebuild_icon_row(
            icons,
            area,
            width,
            self.daily
                .iter()
                .map(|d| get_weather_icon_name(&d.condition, true)),
            "daily-graph-icon",
        );
    }
}

// ---------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------

/// Fixed y-axis: one temperature column. Neutral color, since it serves
/// both the high and low lines.
fn draw_axis(
    area: &gtk::DrawingArea,
    cr: &cairo::Context,
    w: f64,
    h: f64,
    data: &GraphData,
    theme: &GraphTheme,
) {
    if data.points.is_empty() {
        return;
    }
    let g = Geometry::new(h);
    draw_axis_column(
        area,
        cr,
        &g,
        temp_ticks(&g, &data.scale),
        data.unit_label,
        w - AXIS_LABEL_PAD,
        theme.label,
    );
}

/// Scrolling plot: gridlines, high/low band, high and low lines, day labels.
fn draw_plot(
    area: &gtk::DrawingArea,
    cr: &cairo::Context,
    w: f64,
    h: f64,
    data: &GraphData,
    theme: &GraphTheme,
) {
    let pts = &data.points;
    if pts.len() < 2 {
        return;
    }
    let g = Geometry::new(h);
    let scale = &data.scale;
    // Spread the days across the actual width (>= PX_PER_DAY apart).
    let spacing = item_spacing(w, pts.len());
    let day_x = |i: usize| item_x(i, spacing);

    // ---- Grid --------------------------------------------------------------
    draw_horizontal_grid(cr, &g, scale, w, theme);
    // Every day is labeled, so every day gets the stronger line.
    draw_vertical_grid(cr, &g, (0..pts.len()).map(|i| (day_x(i), true)), theme);

    // ---- Curves ------------------------------------------------------------
    let to_xy = |f: fn(&Point) -> f64| -> Vec<(f64, f64)> {
        let points: Vec<(f64, f64)> = pts
            .iter()
            .enumerate()
            .map(|(i, p)| (day_x(i), scale.y(&g, f(p))))
            .collect();
        with_edge_leads(&points, w)
    };
    let high = to_xy(|p| p.high);
    let low = to_xy(|p| p.low);

    // Band between the lines: along the high curve, then back along the low
    // curve. Gradient from the high color (top) to the low color (bottom).
    let low_rev: Vec<(f64, f64)> = low.iter().rev().copied().collect();
    cr.new_path();
    cr.move_to(high[0].0, high[0].1);
    append_smooth(cr, &high);
    cr.line_to(low_rev[0].0, low_rev[0].1);
    append_smooth(cr, &low_rev);
    cr.close_path();

    let (hr, hg, hb, ha) = theme.primary;
    let (lr, lg, lb, la) = theme.secondary;
    let grad = cairo::LinearGradient::new(0.0, g.top, 0.0, g.bottom);
    grad.add_color_stop_rgba(0.0, hr, hg, hb, ha * BAND_ALPHA);
    grad.add_color_stop_rgba(1.0, lr, lg, lb, la * BAND_ALPHA);
    let _ = cr.set_source(&grad);
    let _ = cr.fill();

    stroke_curve(cr, &low, theme.secondary);
    stroke_curve(cr, &high, theme.primary);

    // ---- Day labels --------------------------------------------------------
    draw_bottom_labels(
        area,
        cr,
        &g,
        w,
        pts.iter()
            .enumerate()
            .map(|(i, p)| (day_x(i), p.day_label.as_str())),
        theme.label,
    );
}
