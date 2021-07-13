use crate::commands::{Command, CommandParams, CommandResult};
use crate::errors::HandleUpdateError;
use crate::helpers;

pub const UP: Command = Command {
    name: "up",
    description: "Check bot status",
    is_admin_only: false,
    handler,
    chat_action: None,
};

fn handler(CommandParams { api, message, .. }: CommandParams) -> CommandResult<HandleUpdateError> {
    helpers::send_text_message(
        api,
        message.chat.id,
        "I'm good.".into(),
        Some(message.message_id),
    )
}
