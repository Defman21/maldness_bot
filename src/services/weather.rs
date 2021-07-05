use crate::settings::Settings;
use serde::Deserialize;
use std::{fmt, io};
use ureq::Error;

#[derive(Debug, Deserialize)]
pub struct WeatherResponseMain {
    pub feels_like: f64,
    pub temp: f64,
}

#[derive(Debug, Deserialize)]
pub struct WeatherResponseWeather {
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct WeatherResponse {
    pub name: String,
    pub main: WeatherResponseMain,
    pub weather: Vec<WeatherResponseWeather>,
}

#[derive(Debug)]
pub enum WeatherError {
    Request(Error),
    Json(io::Error),
}

impl fmt::Display for WeatherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Request(ref err) => write!(f, "Request error: {}", err),
            Self::Json(ref err) => write!(f, "Json error: {}", err),
        }
    }
}

impl std::error::Error for WeatherError {}

impl From<Error> for WeatherError {
    fn from(err: Error) -> Self {
        Self::Request(err)
    }
}

impl From<io::Error> for WeatherError {
    fn from(err: io::Error) -> Self {
        Self::Json(err)
    }
}

pub enum Identifier {
    Location(f64, f64),
    Name(String),
}

pub fn get_weather(
    identifier: Identifier,
    settings: &Settings,
) -> Result<WeatherResponse, WeatherError> {
    let mut request = ureq::get("https://api.openweathermap.org/data/2.5/weather")
        .query("units", settings.open_weather.units.as_str())
        .query("lang", settings.open_weather.language.as_str())
        .query("appid", settings.open_weather.api_key.as_str());

    match identifier {
        Identifier::Location(latitude, longitude) => {
            let latitude = latitude.to_string();
            let longitude = longitude.to_string();
            request = request
                .query("lat", latitude.as_str())
                .query("lon", longitude.as_str());
        }
        Identifier::Name(ref query) => {
            request = request.query("q", query.as_str());
        }
    };

    let result: WeatherResponse = request.call()?.into_json()?;

    Ok(result)
}

pub fn format_weather_data(data: &WeatherResponse) -> String {
    let mut text = format!(
        "{}: {} (ощущается как {})",
        data.name, data.main.temp, data.main.feels_like
    );
    if !data.weather.is_empty() {
        let description: String = data
            .weather
            .iter()
            .map(|i| i.description.clone())
            .collect::<Vec<String>>()
            .join(", ");
        text += ", ";
        text += description.as_str();
    };

    text
}
