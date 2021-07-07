use diesel::PgConnection;
use frankenstein::{Api, ChatId, Location, SendMessageParams, TelegramApi, Update};

use crate::commands::{Command, CommandResult};
use crate::errors::HandleUpdateError;
use crate::helpers;
use crate::services::user;
use crate::services::user::functions::User;
use crate::services::weather::{format_weather_data, get_weather, Identifier, WeatherResponse};
use crate::settings::Settings;

pub const WEATHER: Command = Command {
    name: "weather",
    description: "Check the weather at a given location",
    is_admin_only: false,
    handler,
};

fn handler(
    api: &Api,
    update: &Update,
    postgres: &mut PgConnection,
    settings: &Settings,
    args: &str,
) -> CommandResult<HandleUpdateError> {
    let message = update
        .message
        .as_ref()
        .ok_or_else(|| HandleUpdateError::Command("no message".to_string()))?;
    let mut result: Option<WeatherResponse> = None;

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
            )?);
        }
    }

    let user_id = helpers::get_user_id(message)?;

    let send_error_message = || {
        let send_message_params = SendMessageParams::new(
            ChatId::Integer(message.chat.id),
            "You've called the command without any arguments or a reply to a location and \
            you don't have a personal location set! Consider using /set_my_location in reply to a \
            location message to set your location so you could call the command without any arguments".into(),
        );
        if let Some(err) = api.send_message(&send_message_params).err() {
            return HandleUpdateError::Api(err);
        }

        HandleUpdateError::Command("handled return".into())
    };

    if !args.is_empty() {
        result = Some(get_weather(Identifier::Name(args.to_string()), settings)?);
    } else if result.is_none() {
        let User {
            latitude,
            longitude,
            ..
        } = user::functions::get_by_id(postgres, user_id)
            .map_err(|e| HandleUpdateError::Command(e.to_string()))?;
        let latitude = latitude.ok_or_else(send_error_message)?;
        let longitude = longitude.ok_or_else(send_error_message)?;
        result = Some(get_weather(
            Identifier::Location {
                latitude,
                longitude,
            },
            settings,
        )?);
    }

    if let Some(data) = result {
        let mut send_message_params =
            SendMessageParams::new(ChatId::Integer(message.chat.id), format_weather_data(&data));
        send_message_params.set_reply_to_message_id(Some(message.message_id));

        return api
            .send_message(&send_message_params)
            .map(|_| ())
            .map_err(HandleUpdateError::Api);
    }

    Ok(())
}
