use std::str::ParseBoolError;

use crate::commands::{Command, CommandParams, CommandResult};
use crate::errors::HandleUpdateError;
use crate::services::user::functions;

pub const SET_PAYING_STATUS: Command = Command {
    name: "set_paying_status",
    description: "Set the paying status for a user",
    is_admin_only: true,
    handler,
    chat_action: None,
};

fn handler(
    CommandParams {
        conn,
        message,
        args,
        ..
    }: CommandParams,
) -> CommandResult<HandleUpdateError> {
    let is_paying: bool = args
        .parse()
        .map_err(|e: ParseBoolError| HandleUpdateError::Command(e.to_string()))?;

    functions::set_paying_status(conn, message.from.as_ref().unwrap(), is_paying)
        .map(|_| ())
        .map_err(|e| HandleUpdateError::Command(e.to_string()))
}
