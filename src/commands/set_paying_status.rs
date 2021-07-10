use std::str::ParseBoolError;

use diesel::PgConnection;
use frankenstein::{Api, Message, Update};

use crate::commands::{Command, CommandResult};
use crate::errors::HandleUpdateError;
use crate::helpers;
use crate::services::user::functions;
use crate::settings::Settings;

pub const SET_PAYING_STATUS: Command = Command {
    name: "set_paying_status",
    description: "Set the paying status for a user",
    is_admin_only: true,
    handler,
    chat_action: None,
};

fn handler(
    _api: &Api,
    _update: &Update,
    postgres: &mut PgConnection,
    _settings: &Settings,
    message: &Message,
    args: &str,
) -> CommandResult<HandleUpdateError> {
    let is_paying: bool = args
        .parse()
        .map_err(|e: ParseBoolError| HandleUpdateError::Command(e.to_string()))?;

    let user_id = helpers::get_user_id(message)?;

    functions::set_paying_status(postgres, user_id, is_paying)
        .map(|_| ())
        .map_err(|e| HandleUpdateError::Command(e.to_string()))
}
