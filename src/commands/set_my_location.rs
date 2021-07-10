use diesel::PgConnection;
use frankenstein::{Api, Message, Update};

use crate::commands::{Command, CommandResult};
use crate::errors::HandleUpdateError;
use crate::helpers;
use crate::services::user::functions;
use crate::settings::Settings;

pub const SET_MY_LOCATION: Command = Command {
    name: "set_my_location",
    description: "Set the location as my location",
    is_admin_only: false,
    handler,
    chat_action: None,
};

fn handler(
    _api: &Api,
    _update: &Update,
    postgres: &mut PgConnection,
    _settings: &Settings,
    message: &Message,
    _args: &str,
) -> CommandResult<HandleUpdateError> {
    let location = message
        .reply_to_message
        .as_ref()
        .ok_or_else(|| HandleUpdateError::Command("reply to message is empty".into()))?
        .location
        .as_ref()
        .ok_or_else(|| HandleUpdateError::Command("location in the reply is empty".into()))?;

    let user_id = helpers::get_user_id(message)?;

    functions::set_location(postgres, user_id, location.latitude, location.longitude)
        .map(|_| ())
        .map_err(|e| HandleUpdateError::Command(e.to_string()))
}
