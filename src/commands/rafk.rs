use frankenstein::ChatAction;

use crate::commands::{Command, CommandParams, CommandResult};
use crate::errors::HandleUpdateError;

use crate::helpers::send_text_message;
use crate::services::afk_event::functions::EventType;
use crate::services::afk_event::{
    errors::ServiceError as AfkEventServiceError, functions::reset_latest_event,
};
use crate::services::user::errors::ServiceError as UserServiceError;

pub const RAFK: Command = Command {
    name: "rafk",
    description: "Resume your AFK",
    is_admin_only: false,
    handler,
    chat_action: Some(ChatAction::Typing),
};

fn handler(
    CommandParams {
        api,
        cache,
        conn,
        settings,
        message,
        ..
    }: CommandParams,
) -> CommandResult<HandleUpdateError> {
    let user = message.from.as_ref().unwrap();

    let send_no_afk_event_message = || {
        send_text_message(
            api,
            message.chat.id,
            settings
                .no_afk_event_text
                .clone()
                .unwrap_or_else(|| "You haven't been afk tho...".into()),
            Some(message.message_id),
        )
    };

    match reset_latest_event(conn, user) {
        Ok(event) => {
            let text = match event.event_type() {
                EventType::Work => settings
                    .commands
                    .work
                    .work_text
                    .clone()
                    .unwrap_or_else(|| "Have a good one, king.".into()),
                EventType::Sleep => settings
                    .commands
                    .gn
                    .good_night_text
                    .clone()
                    .unwrap_or_else(|| "Good night!".into()),
            };

            cache.cache_afk_event_id(user.id as i64, true, event.id);
            send_text_message(api, message.chat.id, text, Some(message.message_id))
        }
        Err(err) => match err {
            AfkEventServiceError::NotFound
            | AfkEventServiceError::User(UserServiceError::NotFound) => send_no_afk_event_message(),
            err => Err(err.into()),
        },
    }
}
