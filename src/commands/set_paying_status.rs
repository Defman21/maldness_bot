use std::convert::TryFrom;

use frankenstein::{Api, Update};
use postgres::Client;

use crate::commands::Command;
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
) -> Option<HandleUpdateError> {
    let is_paying = match args.parse::<bool>() {
        Ok(val) => val,
        Err(_) => {
            return Some(HandleUpdateError::Command(
                "arg is not a boolean".to_string(),
            ))
        }
    };

    let user_id: i64 = match i64::try_from(
        update
            .message
            .as_ref()?
            .reply_to_message
            .as_ref()?
            .from
            .as_ref()?
            .id,
    ) {
        Ok(val) => val,
        Err(err) => return Some(HandleUpdateError::Command(err.to_string())),
    };

    if let Err(err) = functions::set_paying_status(postgres, user_id, is_paying) {
        return Some(HandleUpdateError::Command(err.to_string()));
    }
    None
}
