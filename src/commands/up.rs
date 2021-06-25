use crate::commands::Command;
use crate::errors::HandleUpdateError;
use frankenstein::{Api, ChatId, SendMessageParams, TelegramApi, Update};

pub const UP: Command = Command {
    name: "up",
    description: "Check bot status",
    handler,
};

fn handler(api: &Api, update: &Update, args: &str) -> Option<HandleUpdateError> {
    let message = update.message.as_ref()?;
    let mut send_message_params = SendMessageParams::new(
        ChatId::Integer(message.chat.id),
        format!("I'm up and running, your args: {:?}", args),
    );
    send_message_params.set_reply_to_message_id(Some(message.message_id));

    if let Some(error) = api.send_message(&send_message_params).err() {
        return Some(HandleUpdateError::Api(error));
    }

    None
}
