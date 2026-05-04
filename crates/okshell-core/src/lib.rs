mod ipc;
mod monitors;
mod relm_app;

use crate::relm_app::{Shell, ShellInit};
use any_spawner::Executor;
use okshell_config::schema::config::{
    ConfigStoreFields, GeneralStoreFields, IconsStoreFields, ThemeStoreFields,
};
use okshell_idle::inhibitor::IdleInhibitor;
use okshell_services::weather_service;
use reactive_graph::effect::Effect;
use reactive_graph::traits::{Get, GetUntracked};
use relm4::prelude::*;
use std::cell::Cell;
use std::error::Error;
use std::sync::OnceLock;
use tokio::runtime::Runtime;
use tracing;
use tracing::info;
use wayle_weather::{LocationQuery, TemperatureUnit};

static TOKIO_RT: OnceLock<Runtime> = OnceLock::new();

fn tokio_rt() -> &'static Runtime {
    TOKIO_RT.get_or_init(|| Runtime::new().expect("tokio runtime"))
}

pub fn run() -> Result<(), Box<dyn Error>> {
    info!("Welcome to OkShell!");

    Executor::init_glib().expect("Executor could not be initialized.");

    let config_manager = okshell_config::config_manager::config_manager();
    config_manager.watch_config();

    // Initialize the effects in the wallpaper store
    let _ = okshell_cache::wallpaper::wallpaper_store();

    let location_query = LocationQuery::from(
        config_manager
            .config()
            .general()
            .weather_location_query()
            .get_untracked(),
    );

    let temperature_units = TemperatureUnit::from(
        config_manager
            .config()
            .general()
            .temperature_unit()
            .get_untracked(),
    );

    tokio_rt().block_on(async {
        okshell_services::init_services(location_query, temperature_units).await
    })?;

    tokio_rt().spawn(async move {
        let _ = IdleInhibitor::global().init().await;
    });

    Effect::new(move |_| {
        let theme = config_manager
            .config()
            .theme()
            .icons()
            .shell_icon_theme()
            .get();
        gtk::Settings::default()
            .unwrap()
            .set_gtk_icon_theme_name(Some(theme.as_str()));
    });

    // skip first run
    let initialized = Cell::new(false);
    Effect::new(move |_| {
        let location_query = config_manager
            .config()
            .general()
            .weather_location_query()
            .get();
        if !initialized.get() {
            initialized.set(true);
            return;
        }
        let weather = weather_service();
        weather.set_location(LocationQuery::from(location_query));
    });

    // skip first run
    let initialized = Cell::new(false);
    Effect::new(move |_| {
        let temp_unit = config_manager.config().general().temperature_unit().get();
        if !initialized.get() {
            initialized.set(true);
            return;
        }
        let weather = weather_service();
        weather.set_units(TemperatureUnit::from(temp_unit));
    });

    let app = RelmApp::new("okshell.main");
    app.run::<Shell>(ShellInit {});

    info!("Goodbye!");

    Ok(())
}
