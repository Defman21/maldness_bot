use frankenstein::{ChatId, SendMessageParams, TelegramApi};

use crate::commands::{Command, CommandParams, CommandResult};
use crate::errors::HandleUpdateError;

pub const UP: Command = Command {
    name: "up",
    description: "Check bot status",
    is_admin_only: false,
    handler,
    chat_action: None,
};

fn handler(
    CommandParams {
        api, message, args, ..
    }: CommandParams,
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
