use frankenstein::{Api, ChatId, SendMessageParams, TelegramApi, Update};
use postgres::Client;

use crate::commands::Command;
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
    update: &Update,
    _postgres: &mut Client,
    settings: &Settings,
    _args: &str,
) -> Option<HandleUpdateError> {
    // TODO: remove clone
    // The problem is that `into_str` takes ownership and we can't take it out of HashMap.
    // One possible solution would be some sort of a cache that we will populate through some proxy
    // call and return from that cache on each call.
    let text = match settings.commands["donate"]["text"].clone().into_str() {
        Ok(str) => str,
        Err(err) => return Some(HandleUpdateError::Command(err.to_string())),
    };
    let message = update.message.as_ref()?;
    let mut send_message_params = SendMessageParams::new(ChatId::Integer(message.chat.id), text);
    send_message_params.set_reply_to_message_id(Some(message.message_id));

    if let Some(error) = api.send_message(&send_message_params).err() {
        return Some(HandleUpdateError::Api(error));
    }

    None
}
