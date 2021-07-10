use diesel::PgConnection;
use frankenstein::{Api, ChatId, Message, SendMessageParams, TelegramApi, Update};
use serde::Deserialize;

use crate::commands::{Command, CommandResult};
use crate::errors::HandleUpdateError;
use crate::settings::Settings;

pub const DONATE: Command = Command {
    name: "donate",
    description: "Support the creator",
    is_admin_only: false,
    handler,
    chat_action: None,
};

#[derive(Debug, Deserialize)]
pub struct CommandSettings {
    pub text: String,
}

fn handler(
    api: &Api,
    _update: &Update,
    _postgres: &mut PgConnection,
    settings: &Settings,
    message: &Message,
    _args: &str,
) -> CommandResult<HandleUpdateError> {
    let text = settings.commands.donate.text.clone();
    let mut send_message_params = SendMessageParams::new(ChatId::Integer(message.chat.id), text);
    send_message_params.set_reply_to_message_id(Some(message.message_id));

    api.send_message(&send_message_params)
        .map(|_| ())
        .map_err(HandleUpdateError::Api)
}
