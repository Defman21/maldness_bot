use frankenstein::{ChatAction, ChatId, SendMessageParams, TelegramApi};
use serde::Deserialize;

use crate::commands::{Command, CommandParams, CommandResult};
use crate::errors::HandleUpdateError;

use crate::helpers;
use crate::services::sleep::functions::SleepType;

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
    crate::services::sleep::functions::go_to_sleep(
        user_id,
        sleep_type,
        Some(args.to_string()),
        conn,
    )?;
    cache.cache_sleep_status(user_id, true);
    let mut send_message_params = SendMessageParams::new(
        ChatId::Integer(message.chat.id),
        settings
            .commands
            .gn
            .good_night_text
            .clone()
            .unwrap_or_else(|| "Good night!".into()),
    );
    send_message_params.set_reply_to_message_id(Some(message.message_id));

    api.send_message(&send_message_params)
        .map(|_| ())
        .map_err(HandleUpdateError::Api)
}
