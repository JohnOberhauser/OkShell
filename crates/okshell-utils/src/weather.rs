use okshell_common::watch;
use okshell_services::weather_service;
use relm4::{Component, ComponentSender};
use wayle_weather::{Speed, Temperature, TemperatureUnit, WeatherCondition};

pub fn get_weather_icon_name(weather_condition: &WeatherCondition, is_day: bool) -> &'static str {
    match weather_condition {
        WeatherCondition::Clear => {
            if is_day {
                "weather-clear-day-symbolic"
            } else {
                "weather-clear-night-symbolic"
            }
        }
        WeatherCondition::PartlyCloudy => {
            if is_day {
                "weather-partly-cloudy-day-symbolic"
            } else {
                "weather-partly-cloudy-night-symbolic"
            }
        }
        WeatherCondition::Cloudy => "weather-cloudy-symbolic",
        WeatherCondition::Overcast => "weather-overcast-symbolic",
        WeatherCondition::Mist => "weather-mist-symbolic",
        WeatherCondition::Fog => "weather-fog-symbolic",
        WeatherCondition::LightRain => "weather-rain-light-symbolic",
        WeatherCondition::Rain => "weather-rain-symbolic",
        WeatherCondition::HeavyRain => "weather-rain-heavy-symbolic",
        WeatherCondition::Drizzle => "weather-drizzle-symbolic",
        WeatherCondition::LightSnow => "weather-snow-light-symbolic",
        WeatherCondition::Snow => "weather-snow-symbolic",
        WeatherCondition::HeavySnow => "weather-snow-heavy-symbolic",
        WeatherCondition::Sleet => "weather-sleet-symbolic",
        WeatherCondition::Thunderstorm => "weather-thunderstorm-symbolic",
        WeatherCondition::Windy => "weather-windy-symbolic",
        WeatherCondition::Hail => "weather-hail-symbolic",
        WeatherCondition::Unknown => "weather-unknown-symbolic",
    }
}

pub fn get_temperature_string(
    temperature: &Temperature,
    temperature_unit: &TemperatureUnit,
) -> String {
    match temperature_unit {
        TemperatureUnit::Metric => {
            format!("{}°C", temperature.celsius())
        }
        TemperatureUnit::Imperial => {
            format!("{}°F", temperature.fahrenheit())
        }
    }
}

pub fn get_wind_speed(wind_speed: &Speed, temperature_unit: &TemperatureUnit) -> String {
    match temperature_unit {
        TemperatureUnit::Metric => wind_speed.kmh().to_string(),
        TemperatureUnit::Imperial => wind_speed.mph().to_string(),
    }
}

pub fn get_wind_speed_units_string(temperature_unit: &TemperatureUnit) -> &'static str {
    match temperature_unit {
        TemperatureUnit::Metric => " kmh winds",
        TemperatureUnit::Imperial => " mph winds",
    }
}

pub fn spawn_weather_watcher<C>(
    sender: &ComponentSender<C>,
    status_state: impl Fn() -> C::CommandOutput + Send + Sync + 'static,
) where
    C: Component,
    C::CommandOutput: Send + 'static,
{
    let service = weather_service();

    let status = service.status.clone();
    let weather = service.weather.clone();

    watch!(sender, [status.watch(), weather.watch()], |out| {
        let _ = out.send(status_state());
    });
}
