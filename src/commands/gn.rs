use frankenstein::ChatAction;
use serde::Deserialize;

use crate::commands::{Command, CommandParams, CommandResult};
use crate::errors::HandleUpdateError;

use crate::helpers;
use crate::services::sleep::functions::{go_to_sleep, SleepType};

pub const GOOD_NIGHT: Command = Command {
    name: "gn",
    description: "Good night, sweet prince!",
    is_admin_only: false,
    handler,
    chat_action: Some(ChatAction::Typing),
};

#[derive(Debug, Deserialize)]
pub struct CommandSettings {
    pub good_night_text: Option<String>,
}

fn handler(
    CommandParams {
        api,
        conn,
        settings,
        cache,
        message,
        args,
        ..
    }: CommandParams,
) -> CommandResult<HandleUpdateError> {
    let user_id = helpers::get_user_id(message)?;
    let sleep_type = match args {
        "rafk" => SleepType::Continue,
        _ => SleepType::New,
    };
    let afk_message = match args.is_empty() {
        true => None,
        false => Some(args.to_string()),
    };
    go_to_sleep(user_id, sleep_type, afk_message, conn)?;
    cache.cache_sleep_status(user_id, true);
    helpers::send_text_message(
        api,
        message.chat.id,
        settings
            .commands
            .gn
            .good_night_text
            .clone()
            .unwrap_or_else(|| "Good night!".into()),
        Some(message.message_id),
    )
}
