use frankenstein::ChatAction;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Deserialize;

use crate::commands::{Command, CommandParams, CommandResult};
use crate::errors::HandleUpdateError;
use crate::helpers;

pub const SHUFFLE: Command = Command {
    name: "shuffle",
    description: "Shuffle words",
    is_admin_only: false,
    handler,
    chat_action: Some(ChatAction::Typing),
};

#[derive(Debug, Deserialize)]
pub struct CommandSettings {
    pub nothing_to_shuffle_text: Option<String>,
}

fn handler(
    CommandParams {
        api,
        message,
        settings,
        args,
        ..
    }: CommandParams,
) -> CommandResult<HandleUpdateError> {
    let nothing_to_shuffle = settings
        .commands
        .shuffle
        .nothing_to_shuffle_text
        .clone()
        .unwrap_or_else(|| "Nothing to shuffle!".into());
    let text: Option<String>;

    if !args.is_empty() {
        text = Some(args.to_string());
    } else {
        text = message
            .reply_to_message
            .as_ref()
            .and_then(|reply| reply.text.clone());
    }

    if text.is_none() {
        return helpers::send_text_message(
            api,
            message.chat.id,
            nothing_to_shuffle,
            Some(message.message_id),
        );
    }

    let text = text.unwrap();

    let mut rng = thread_rng();
    let mut words: Vec<&str> = text.split(' ').collect();
    words.shuffle(&mut rng);

    helpers::send_text_message(
        api,
        message.chat.id,
        words.join(" "),
        Some(message.message_id),
    )
}
