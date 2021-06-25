use crate::commands::Command;
use crate::errors::HandleUpdateError;
use frankenstein::{Api, ChatId, SendMessageParams, TelegramApi, Update};
use std::env;

pub const DONATE: Command = Command {
    name: "donate",
    description: "Support the creator",
    handler,
};

fn handler(api: &Api, update: &Update, _args: &str) -> Option<HandleUpdateError> {
    let message = update.message.as_ref()?;
    let text = match env::var("DONATE_TEXT") {
        Ok(val) => val,
        Err(err) => return Some(HandleUpdateError::Command(err.to_string())),
    };
    let mut send_message_params = SendMessageParams::new(ChatId::Integer(message.chat.id), text);
    send_message_params.set_reply_to_message_id(Some(message.message_id));

    if let Some(error) = api.send_message(&send_message_params).err() {
        return Some(HandleUpdateError::Api(error));
    }

    None
}
