use frankenstein::{ChatId, SendMessageParams, TelegramApi};
use serde::Deserialize;

use crate::commands::{Command, CommandParams, CommandResult};
use crate::errors::HandleUpdateError;

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
    CommandParams {
        api,
        settings,
        message,
        ..
    }: CommandParams,
) -> CommandResult<HandleUpdateError> {
    let text = settings.commands.donate.text.clone();
    let mut send_message_params = SendMessageParams::new(ChatId::Integer(message.chat.id), text);
    send_message_params.set_reply_to_message_id(Some(message.message_id));

    api.send_message(&send_message_params)
        .map(|_| ())
        .map_err(HandleUpdateError::Api)
}
