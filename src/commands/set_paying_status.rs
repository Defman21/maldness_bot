use std::convert::TryFrom;
use std::str::ParseBoolError;

use frankenstein::{Api, Update};
use postgres::Client;

use crate::commands::{Command, CommandResult};
use crate::errors::HandleUpdateError;
use crate::services::user::functions;
use crate::settings::Settings;

pub const SET_PAYING_STATUS: Command = Command {
    name: "set_paying_status",
    description: "Set the paying status for a user",
    is_admin_only: true,
    handler,
};

fn handler(
    _api: &Api,
    update: &Update,
    postgres: &mut Client,
    _settings: &Settings,
    args: &str,
) -> CommandResult<HandleUpdateError> {
    let is_paying: bool = args
        .parse()
        .map_err(|e: ParseBoolError| HandleUpdateError::Command(e.to_string()))?;

    let user_id: i64 = i64::try_from(
        update
            .message
            .as_ref()
            .ok_or_else(|| HandleUpdateError::Command("message is empty".to_string()))?
            .reply_to_message
            .as_ref()
            .ok_or_else(|| HandleUpdateError::Command("no reply for the message".to_string()))?
            .from
            .as_ref()
            .ok_or_else(|| HandleUpdateError::Command("no from data".to_string()))?
            .id,
    )
        .map_err(|e| HandleUpdateError::Command(e.to_string()))?;

    functions::set_paying_status(postgres, user_id, is_paying)
        .map(|_| ())
        .map_err(|e| HandleUpdateError::Command(e.to_string()))
}
