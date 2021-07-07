use diesel::PgConnection;
use frankenstein::{Api, ChatId, SendMessageParams, TelegramApi, Update, Message};

use crate::commands::{Command, CommandResult};
use crate::errors::HandleUpdateError;
use crate::settings::Settings;

pub const DONATE: Command = Command {
    name: "donate",
    description: "Support the creator",
    is_admin_only: false,
    handler,
};

fn handler(
    api: &Api,
    _update: &Update,
    _postgres: &mut PgConnection,
    settings: &Settings,
    message: &Message,
    _args: &str,
) -> CommandResult<HandleUpdateError> {
    // TODO: remove clone
    // The problem is that `into_str` takes ownership and we can't take it out of HashMap.
    // One possible solution would be some sort of a cache that we will populate through some proxy
    // call and return from that cache on each call.
    let text = settings.commands["donate"]["text"]
        .clone()
        .into_str()
        .map_err(|e| HandleUpdateError::Command(e.to_string()))?;
    let mut send_message_params = SendMessageParams::new(ChatId::Integer(message.chat.id), text);
    send_message_params.set_reply_to_message_id(Some(message.message_id));

    api.send_message(&send_message_params)
        .map(|_| ())
        .map_err(HandleUpdateError::Api)
}
