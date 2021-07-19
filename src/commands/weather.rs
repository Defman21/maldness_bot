use frankenstein::ChatAction;
use serde::Deserialize;
use ureq::Error as RequestError;

use crate::commands::{Command, CommandParams, CommandResult};
use crate::errors::HandleUpdateError;
use crate::helpers;
use crate::services::user;
use crate::services::user::errors::ServiceError;
use crate::services::user::functions::User;
use crate::services::weather::{
    format_weather_data, get_weather, Identifier, WeatherError, WeatherResponse,
};

pub const WEATHER: Command = Command {
    name: "weather",
    description: "Check the weather at a given location",
    is_admin_only: false,
    handler,
    chat_action: Some(ChatAction::FindLocation),
};

#[derive(Debug, Deserialize)]
pub struct CommandSettings {
    not_found_text: Option<String>,
    no_location_text: Option<String>,
    no_location_for_user_text: Option<String>,
}

fn handler(
    CommandParams {
        api,
        conn,
        settings,
        message,
        args,
        ..
    }: CommandParams,
) -> CommandResult<HandleUpdateError> {
    let result: Result<WeatherResponse, WeatherError>;

    let from = message.from.as_ref().unwrap();

    let send_error_message = |user: &frankenstein::User| {
        let text = match user.id == from.id {
            true => settings
                .commands
                .weather
                .no_location_text
                .clone()
                .unwrap_or_else(|| {
                    "You don't have a location set. Send me a geolocation message \
                    and call /set_my_location on it."
                        .into()
                }),
            false => settings
                .commands
                .weather
                .no_location_for_user_text
                .clone()
                .unwrap_or_else(|| "This user does not have a location set.".into()),
        };

        if let Err(err) =
            helpers::send_text_message(api, message.chat.id, text, Some(message.message_id))
        {
            return err;
        }
        HandleUpdateError::Command("handled return".into())
    };

    let mut get_location_by_user = |user: &frankenstein::User| -> Result<
        Result<WeatherResponse, WeatherError>,
        HandleUpdateError,
    > {
        let User {
            latitude,
            longitude,
            ..
        } = user::functions::get_by_telegram_user(conn, user).map_err(|err| match err {
            ServiceError::NotFound => send_error_message(user),
            err => HandleUpdateError::Command(err.to_string()),
        })?;
        let latitude = latitude.ok_or_else(|| send_error_message(user))?;
        let longitude = longitude.ok_or_else(|| send_error_message(user))?;

        Ok(get_weather(
            Identifier::Location {
                latitude,
                longitude,
            },
            settings,
        ))
    };

    if !args.is_empty() {
        result = get_weather(Identifier::Name(args.to_string()), settings);
    } else if let Some(reply) = message.reply_to_message.as_ref() {
        if let Some(location) = reply.location.as_ref() {
            result = get_weather(
                Identifier::Location {
                    latitude: location.latitude,
                    longitude: location.longitude,
                },
                settings,
            );
        } else {
            result = get_location_by_user(reply.from.as_ref().unwrap())?;
        }
    } else {
        result = get_location_by_user(from)?;
    }

    match result {
        Ok(ref data) => helpers::send_text_message(
            api,
            message.chat.id,
            format_weather_data(&data, &settings),
            Some(message.message_id),
        ),
        Err(err) => match err {
            WeatherError::Json(io_err) => Err(HandleUpdateError::Command(io_err.to_string())),
            WeatherError::Request(http_err) => match http_err {
                RequestError::Status(404, _) => helpers::send_text_message(
                    api,
                    message.chat.id,
                    settings
                        .commands
                        .weather
                        .not_found_text
                        .clone()
                        .unwrap_or_else(|| "No weather data for this location found".into()),
                    Some(message.message_id),
                ),
                err => Err(HandleUpdateError::Command(err.to_string())),
            },
        },
    }
}
