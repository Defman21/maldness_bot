use diesel::PgConnection;
use frankenstein::{
    Api, ChatAction, ChatId, Message, SendMessageParams, TelegramApi, Update,
};
use serde::Deserialize;
use ureq::Error as RequestError;

use crate::commands::{Command, CommandResult};
use crate::errors::HandleUpdateError;
use crate::helpers;
use crate::services::user;
use crate::services::user::errors::ServiceError;
use crate::services::user::functions::User;
use crate::services::weather::{
    format_weather_data, get_weather, Identifier, WeatherError, WeatherResponse,
};
use crate::settings::Settings;

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
    api: &Api,
    _update: &Update,
    postgres: &mut PgConnection,
    settings: &Settings,
    message: &Message,
    args: &str,
) -> CommandResult<HandleUpdateError> {
    let result: Result<WeatherResponse, WeatherError>;

    let from_id = helpers::get_user_id(message)?;

    let send_error_message = |user_id: i64| {
        let text = match user_id == from_id {
            true => {
                settings.commands.weather.no_location_text.clone().unwrap_or_else(|| {
                    "You don't have a location set. Send me a geolocation message \
                    and call /set_my_location on it.".into()
                })
            }
            false => {
                settings.commands.weather.no_location_for_user_text.clone().unwrap_or_else(|| {
                    "This user does not have a location set.".into()
                })
            }
        };

        let send_message_params = SendMessageParams::new(
            ChatId::Integer(message.chat.id),
            text,
        );
        if let Some(err) = api.send_message(&send_message_params).err() {
            return HandleUpdateError::Api(err);
        }

        HandleUpdateError::Command("handled return".into())
    };

    let mut get_location_by_user = |user_id: i64| -> Result<Result<WeatherResponse, WeatherError>, HandleUpdateError>{
        let User {
            latitude,
            longitude,
            ..
        } = user::functions::get_by_id(postgres, user_id).map_err(|err| {
            match err {
                ServiceError::NotFound => send_error_message(user_id),
                err => HandleUpdateError::Command(err.to_string()),
            }
        })?;
        let latitude = latitude.ok_or_else(|| send_error_message(user_id))?;
        let longitude = longitude.ok_or_else(|| send_error_message(user_id))?;

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
            let user_id = helpers::get_user_id(reply)?;
            result = get_location_by_user(user_id)?;
        }
    } else {
        result = get_location_by_user(from_id)?;
    }

    match result {
        Ok(ref data) => {
            let mut send_message_params = SendMessageParams::new(
                ChatId::Integer(message.chat.id),
                format_weather_data(&data, &settings),
            );
            send_message_params.set_reply_to_message_id(Some(message.message_id));

            api.send_message(&send_message_params)
                .map(|_| ())
                .map_err(HandleUpdateError::Api)
        }
        Err(err) => match err {
            WeatherError::Json(io_err) => Err(HandleUpdateError::Command(io_err.to_string())),
            WeatherError::Request(http_err) => match http_err {
                RequestError::Status(404, _) => {
                    let mut send_message_params = SendMessageParams::new(
                        ChatId::Integer(message.chat.id),
                        settings
                            .commands
                            .weather
                            .not_found_text
                            .clone()
                            .unwrap_or_else(|| "No weather data for this location found".into()),
                    );
                    send_message_params.set_reply_to_message_id(Some(message.message_id));

                    api.send_message(&send_message_params)
                        .map(|_| ())
                        .map_err(HandleUpdateError::Api)
                }
                err => Err(HandleUpdateError::Command(err.to_string())),
            },
        },
    }
}
