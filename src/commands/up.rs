use diesel::PgConnection;
use frankenstein::{Api, ChatId, Message, SendMessageParams, TelegramApi, Update};

use crate::commands::{Command, CommandResult};
use crate::errors::HandleUpdateError;
use crate::settings::Settings;

pub const UP: Command = Command {
    name: "up",
    description: "Check bot status",
    is_admin_only: false,
    handler,
};

fn handler(
    api: &Api,
    _update: &Update,
    _postgres: &mut PgConnection,
    _settings: &Settings,
    message: &Message,
    args: &str,
) -> CommandResult<HandleUpdateError> {
    let mut send_message_params = SendMessageParams::new(
        ChatId::Integer(message.chat.id),
        format!("I'm up and running, your args: {:?}", args),
    );
    send_message_params.set_reply_to_message_id(Some(message.message_id));

    api.send_message(&send_message_params)
        .map(|_| ())
        .map_err(HandleUpdateError::Api)
}
