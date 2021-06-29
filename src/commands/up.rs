use frankenstein::{Api, ChatId, SendMessageParams, TelegramApi, Update};
use postgres::Client;

use crate::commands::Command;
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
    update: &Update,
    _postgres: &mut Client,
    _settings: &Settings,
    args: &str,
) -> Option<HandleUpdateError> {
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
