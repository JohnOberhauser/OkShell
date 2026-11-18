use std::cell::RefCell;
use std::rc::Rc;

use crate::menus::menu_widgets::weather::graph_common::{
    AXIS_LABEL_PAD, GRAPH_HEIGHT, Geometry, GraphTheme, ICON_ROW_HEIGHT, ICON_SIZE, TempScale,
    Y_DIVISIONS, content_width, draw_axis_column, draw_bottom_labels, draw_horizontal_grid,
    draw_vertical_grid, follow_plot_width, item_spacing, item_x, rebuild_icon_row,
    rounded_top_rect, set_color, set_themed_draw_func, smooth_path, stroke_curve, temp_ticks,
    temp_unit_label, with_edge_leads,
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
use wayle_weather::{HourlyForecast, TemperatureUnit};

/// Horizontal space per hour.
const PX_PER_HOUR: i32 = 22;
/// Label the x-axis on hours divisible by this (clock-aligned: 12, 4, 8, ...).
const LABEL_EVERY_HOURS: u32 = 4;

/// Fixed (non-scrolling) y-axis: UV column, then temperature column, then the plot.
const UV_AXIS_WIDTH: f64 = 34.0;
const TEMP_AXIS_WIDTH: f64 = 34.0;
/// Space between the UV labels and the temp column (gap between the two axes).
const AXIS_COLUMN_GAP: f64 = 12.0;
const AXIS_WIDTH: i32 = (UV_AXIS_WIDTH + TEMP_AXIS_WIDTH) as i32;

/// UV scale top. Higher readings are clamped to the top.
/// UV_TICKS must have Y_DIVISIONS + 1 entries so they share gridlines with temp.
const UV_SCALE_MAX: f64 = 12.0;
const UV_TICKS: [f64; Y_DIVISIONS + 1] = [0.0, 4.0, 8.0, 12.0];

struct Point {
    temp: f64,
    uv: f64,
    /// Only Some on every LABEL_EVERY_HOURS-th hour.
    hour_label: Option<String>,
}

#[derive(Default)]
struct GraphData {
    points: Vec<Point>,
    scale: TempScale,
    /// "°C" / "°F", shown under the temp axis column.
    unit_label: &'static str,
}

fn uv_y(g: &Geometry, uv: f64) -> f64 {
    g.y(uv.clamp(0.0, UV_SCALE_MAX), 0.0, UV_SCALE_MAX)
}

pub(crate) struct HourlyGraphModel {
    hourly: Vec<HourlyForecast>,
    temperature_unit: TemperatureUnit,
    format_24_h: bool,
    /// Shared with both draw funcs. Rebuilt on input, read on every draw.
    data: Rc<RefCell<GraphData>>,
    _effects: EffectScope,
}

#[derive(Debug)]
pub(crate) enum HourlyGraphInput {
    Update(Vec<HourlyForecast>),
    UpdateTemperatureUnit(TemperatureUnit),
    ChangeFormat(bool),
}

#[derive(Debug)]
pub(crate) enum HourlyGraphOutput {}

pub(crate) struct HourlyGraphInit {
    pub hourly: Vec<HourlyForecast>,
}

#[derive(Debug)]
pub(crate) enum HourlyGraphCommandOutput {}

#[relm4::component(pub)]
impl Component for HourlyGraphModel {
    type CommandOutput = HourlyGraphCommandOutput;
    type Input = HourlyGraphInput;
    type Output = HourlyGraphOutput;
    type Init = HourlyGraphInit;

    view! {
        #[root]
        gtk::Box {
            add_css_class: "weather-container",
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,

            gtk::Label {
                add_css_class: "label-large-bold",
                set_label: "Hourly Conditions",
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
                        add_css_class: "hourly-graph",
                        add_css_class: "hourly-graph-axis",
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
                            add_css_class: "hourly-graph-icons",
                            set_height_request: ICON_ROW_HEIGHT,
                        },

                        #[name = "area"]
                        gtk::DrawingArea {
                            add_css_class: "hourly-graph",
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
            sender_clone.input(HourlyGraphInput::UpdateTemperatureUnit(
                TemperatureUnit::from(unit),
            ));
        });

        let config = base_config.clone();
        let sender_clone = sender.clone();
        effects.push(move |_| {
            let config = config.clone();
            let format_24_h = config.general().clock_format_24_h().get();
            sender_clone.input(HourlyGraphInput::ChangeFormat(format_24_h));
        });

        let config = base_config.clone();
        let temperature_unit = config.general().temperature_unit().get_untracked();
        let config = base_config.clone();
        let format_24_h = config.general().clock_format_24_h().get_untracked();

        let model = HourlyGraphModel {
            hourly: params.hourly,
            temperature_unit: TemperatureUnit::from(temperature_unit),
            format_24_h,
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
            HourlyGraphInput::Update(hourly) => self.hourly = hourly,
            HourlyGraphInput::UpdateTemperatureUnit(unit) => self.temperature_unit = unit,
            HourlyGraphInput::ChangeFormat(format_24_h) => self.format_24_h = format_24_h,
        }

        self.rebuild(&widgets.area, &widgets.axis, &widgets.icons);
        self.update_view(widgets, sender);
    }
}

impl HourlyGraphModel {
    /// Convert forecasts into plot-ready points, resize, and queue redraws.
    fn rebuild(&self, area: &gtk::DrawingArea, axis: &gtk::DrawingArea, icons: &gtk::Fixed) {
        let unit = &self.temperature_unit;
        let points: Vec<Point> = self
            .hourly
            .iter()
            .map(|h| Point {
                temp: get_temperature_value(&h.temperature, unit) as f64,
                uv: h.uv_index.get() as f64,
                hour_label: (hour_of(h) % LABEL_EVERY_HOURS == 0).then(|| {
                    if self.format_24_h {
                        h.time.format("%H").to_string()
                    } else {
                        h.time.format("%-I %p").to_string()
                    }
                }),
            })
            .collect();

        let scale = TempScale::fit(points.iter().map(|p| p.temp));

        *self.data.borrow_mut() = GraphData {
            points,
            scale,
            unit_label: temp_unit_label(unit),
        };

        let width = content_width(self.hourly.len(), PX_PER_HOUR);
        area.set_content_width(width);
        area.queue_draw();
        axis.queue_draw();

        rebuild_icon_row(
            icons,
            area,
            width,
            self.hourly
                .iter()
                .map(|h| get_weather_icon_name(&h.condition, h.is_day)),
            "hourly-graph-icon",
        );
    }
}

/// Hour of day (0-23), derived through the same `format` call the rest of the
/// weather widgets already use, so no extra time-crate trait imports are needed.
/// If `h.time` is a `time::OffsetDateTime`, you can replace this with `h.time.hour() as u32`.
fn hour_of(h: &HourlyForecast) -> u32 {
    h.time.format("%H").to_string().parse().unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------

/// Fixed y-axis: UV column, then temperature column. Each tinted with its
/// series color so it's clear which is which.
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

    let uv_ticks = UV_TICKS.map(|uv| (uv_y(&g, uv), format!("{uv:.0}")));
    draw_axis_column(
        area,
        cr,
        &g,
        uv_ticks,
        "UV",
        UV_AXIS_WIDTH - AXIS_COLUMN_GAP,
        theme.secondary,
    );

    draw_axis_column(
        area,
        cr,
        &g,
        temp_ticks(&g, &data.scale),
        data.unit_label,
        w - AXIS_LABEL_PAD,
        theme.primary,
    );
}

/// Scrolling plot: gridlines, UV bars, temp curve, hour labels.
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
    // Spread the hours across the actual width (>= PX_PER_HOUR apart).
    let spacing = item_spacing(w, pts.len());
    let hour_x = |i: usize| item_x(i, spacing);

    // ---- Grid --------------------------------------------------------------
    draw_horizontal_grid(cr, &g, scale, w, theme);
    // Labeled hours get the stronger line.
    draw_vertical_grid(
        cr,
        &g,
        pts.iter()
            .enumerate()
            .map(|(i, p)| (hour_x(i), p.hour_label.is_some())),
        theme,
    );

    // ---- UV bars (behind the temperature curve) ----------------------------
    // Grows with the spacing, but capped so bars stay bar-shaped on wide layouts.
    let bw = (spacing * 0.6).min(ICON_SIZE as f64);
    set_color(cr, theme.secondary, 0.5);
    for (i, p) in pts.iter().enumerate() {
        let y = uv_y(&g, p.uv);
        let bh = g.bottom - y;
        if bh < 1.0 {
            continue;
        }
        let left = (hour_x(i) - bw / 2.0).max(0.0);
        let right = (hour_x(i) + bw / 2.0).min(w);
        rounded_top_rect(cr, left, y, right - left, bh, 2.0);
        let _ = cr.fill();
    }

    // ---- Temperature curve (on top) ----------------------------------------
    let points: Vec<(f64, f64)> = pts
        .iter()
        .enumerate()
        .map(|(i, p)| (hour_x(i), scale.y(&g, p.temp)))
        .collect();
    let xy = with_edge_leads(&points, w);
    let (first, last) = (xy[0], xy[xy.len() - 1]);

    // Light gradient fill — kept faint so the UV bars stay visible through it.
    let (r, gr, b, a) = theme.primary;
    smooth_path(cr, &xy);
    cr.line_to(last.0, g.bottom);
    cr.line_to(first.0, g.bottom);
    cr.close_path();
    let grad = cairo::LinearGradient::new(0.0, g.top, 0.0, g.bottom);
    grad.add_color_stop_rgba(0.0, r, gr, b, a * 0.18);
    grad.add_color_stop_rgba(1.0, r, gr, b, 0.0);
    let _ = cr.set_source(&grad);
    let _ = cr.fill();

    stroke_curve(cr, &xy, theme.primary);

    // ---- Hour labels (every LABEL_EVERY_HOURS) -----------------------------
    draw_bottom_labels(
        area,
        cr,
        &g,
        w,
        pts.iter()
            .enumerate()
            .filter_map(|(i, p)| p.hour_label.as_deref().map(|l| (hour_x(i), l))),
        theme.label,
    );
}
