use crate::commands::Command;
use crate::errors::HandleUpdateError;
use frankenstein::{Api, Update};
use postgres::Client;

pub const SET_PAYING_STATUS: Command = Command {
    name: "set_paying_status",
    description: "Set paying status for a user",
    is_admin_only: true,
    handler,
};

fn handler(_api: &Api, _update: &Update, _postgres: &mut Client, _args: &str) -> Option<HandleUpdateError> {
    None
}
