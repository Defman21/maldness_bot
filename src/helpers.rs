use std::convert::TryFrom;

use frankenstein::{Api, ChatId, Message, SendMessageParams, TelegramApi};

use crate::commands::CommandResult;
use crate::errors::HandleUpdateError;

pub fn get_user_id_by_message(message: &Message) -> Result<i64, HandleUpdateError> {
    i64::try_from(
        message
            .from
            .as_ref()
            .ok_or_else(|| HandleUpdateError::Command("from in the message is empty".into()))?
            .id,
    )
    .map_err(|e| HandleUpdateError::Command(e.to_string()))
}

pub fn send_text_message(
    api: &Api,
    chat_id: i64,
    text: String,
    reply_to_message_id: Option<i32>,
) -> CommandResult<HandleUpdateError> {
    let mut send_message_params = SendMessageParams::new(ChatId::Integer(chat_id), text);
    send_message_params.set_reply_to_message_id(reply_to_message_id);

    api.send_message(&send_message_params)
        .map(|_| ())
        .map_err(HandleUpdateError::Api)
}
