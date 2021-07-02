use frankenstein::{Api, ChatId, SendMessageParams, TelegramApi, Update};
use postgres::Client;
use serde::Deserialize;

use crate::commands::{Command, CommandResult};
use crate::errors::HandleUpdateError;
use crate::settings::Settings;

pub const WEATHER: Command = Command {
    name: "weather",
    description: "Check the weather at a given location",
    is_admin_only: false,
    handler,
};

#[derive(Debug, Deserialize)]
struct WeatherResponseMain {
    feels_like: f64,
    temp: f64,
}

#[derive(Debug, Deserialize)]
struct WeatherResponseWeather {
    description: String,
}

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    name: String,
    main: WeatherResponseMain,
    weather: Vec<WeatherResponseWeather>,
}

fn handler(
    api: &Api,
    update: &Update,
    _postgres: &mut Client,
    settings: &Settings,
    args: &str,
) -> CommandResult<HandleUpdateError> {
    let message = update
        .message
        .as_ref()
        .ok_or_else(|| HandleUpdateError::Command("no message".to_string()))?;
    let mut result: Option<WeatherResponse> = None;

    if let Some(reply) = message.reply_to_message.as_ref() {
        if let Some(location) = reply.location.as_ref() {
            let latitude = location.latitude.to_string();
            let longitude = location.longitude.to_string();
            result = Some(
                ureq::get("https://api.openweathermap.org/data/2.5/weather")
                    .query("lat", latitude.as_str())
                    .query("lon", longitude.as_str())
                    .query("units", settings.open_weather.units.as_str())
                    .query("lang", settings.open_weather.language.as_str())
                    .query("appid", settings.open_weather.api_key.as_str())
                    .call()
                    .map_err(|e| HandleUpdateError::Command(e.to_string()))?
                    .into_json::<WeatherResponse>()
                    .map_err(|e| HandleUpdateError::Command(e.to_string()))?,
            );
        }
    }

    if !args.is_empty() {
        result = Some(
            ureq::get("https://api.openweathermap.org/data/2.5/weather")
                .query("q", args)
                .query("units", settings.open_weather.units.as_str())
                .query("lang", settings.open_weather.language.as_str())
                .query("appid", settings.open_weather.api_key.as_str())
                .call()
                .map_err(|e| HandleUpdateError::Command(e.to_string()))?
                .into_json::<WeatherResponse>()
                .map_err(|e| HandleUpdateError::Command(e.to_string()))?,
        )
    }

    if let Some(data) = result {
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
        }

        let mut send_message_params =
            SendMessageParams::new(ChatId::Integer(message.chat.id), text);
        send_message_params.set_reply_to_message_id(Some(message.message_id));

        return api
            .send_message(&send_message_params)
            .map(|_| ())
            .map_err(HandleUpdateError::Api);
    }

    Ok(())
}
