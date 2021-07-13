use serde::Deserialize;

use crate::commands::{Command, CommandParams, CommandResult};
use crate::errors::HandleUpdateError;
use crate::helpers;

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
    helpers::send_text_message(api, message.chat.id, text, Some(message.message_id))
}
