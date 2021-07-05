use frankenstein::{Api, ChatId, SendMessageParams, TelegramApi, Update};
use postgres::Client;

use crate::commands::{Command, CommandResult};
use crate::errors::HandleUpdateError;
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
            result = Some(get_weather(
                Identifier::Location(location.latitude, location.longitude),
                settings,
            )?);
        }
    }

    if !args.is_empty() {
        result = Some(get_weather(Identifier::Name(args.to_string()), settings)?);
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
