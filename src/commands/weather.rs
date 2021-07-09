use diesel::PgConnection;
use frankenstein::{Api, ChatId, Location, Message, SendMessageParams, TelegramApi, Update};
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
};

#[derive(Debug, Deserialize)]
pub struct CommandSettings {
    not_found_text: Option<String>,
    no_location_text: Option<String>,
}

fn handler(
    api: &Api,
    _update: &Update,
    postgres: &mut PgConnection,
    settings: &Settings,
    message: &Message,
    args: &str,
) -> CommandResult<HandleUpdateError> {
    let mut result: Option<Result<WeatherResponse, WeatherError>> = None;

    if let Some(reply) = message.reply_to_message.as_ref() {
        if let Some(Location {
            latitude,
            longitude,
            ..
        }) = reply.location
        {
            result = Some(get_weather(
                Identifier::Location {
                    latitude,
                    longitude,
                },
                settings,
            ));
        }
    }

    let user_id = helpers::get_user_id(message)?;

    let send_error_message = || {
        let send_message_params = SendMessageParams::new(
            ChatId::Integer(message.chat.id),
            settings.commands.weather.no_location_text.clone().unwrap_or_else(|| {
                "You've called the command without any arguments or a reply to a location and \
                you don't have a personal location set! Consider using /set_my_location in reply to a \
                location message to set your location so you could call the command without any arguments".into()
            })
        );
        if let Some(err) = api.send_message(&send_message_params).err() {
            return HandleUpdateError::Api(err);
        }

        HandleUpdateError::Command("handled return".into())
    };

    if !args.is_empty() {
        result = Some(get_weather(Identifier::Name(args.to_string()), settings));
    } else if result.is_none() {
        let User {
            latitude,
            longitude,
            ..
        } = user::functions::get_by_id(postgres, user_id).map_err(|e| match e {
            ServiceError::NotFound => send_error_message(),
            _ => HandleUpdateError::Command(e.to_string()),
        })?;
        let latitude = latitude.ok_or_else(send_error_message)?;
        let longitude = longitude.ok_or_else(send_error_message)?;
        result = Some(get_weather(
            Identifier::Location {
                latitude,
                longitude,
            },
            settings,
        ));
    }

    match result.unwrap() {
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
                            .unwrap_or_else(|| "City not found".into()),
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
