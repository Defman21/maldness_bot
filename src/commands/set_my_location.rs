use crate::commands::{Command, CommandParams, CommandResult};
use crate::errors::HandleUpdateError;
use crate::helpers;
use crate::services::user::functions;

pub const SET_MY_LOCATION: Command = Command {
    name: "set_my_location",
    description: "Set the location as my location",
    is_admin_only: false,
    handler,
    chat_action: None,
};

fn handler(
    CommandParams {
        api, conn, message, ..
    }: CommandParams,
) -> CommandResult<HandleUpdateError> {
    let location = message
        .reply_to_message
        .as_ref()
        .ok_or_else(|| HandleUpdateError::Command("reply to message is empty".into()))?
        .location
        .as_ref()
        .ok_or_else(|| HandleUpdateError::Command("location in the reply is empty".into()))?;

    functions::set_location(
        conn,
        message.from.as_ref().unwrap(),
        location.latitude,
        location.longitude,
    )
    .map(|_| ())
    .map_err(|e| HandleUpdateError::Command(e.to_string()))?;

    // todo: replace "Location set" with a setting
    helpers::send_text_message(
        api,
        message.chat.id,
        "Location set!".into(),
        Some(message.message_id),
    )
}
