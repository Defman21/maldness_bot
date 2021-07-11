use std::convert::TryFrom;

use frankenstein::Message;

use crate::errors::HandleUpdateError;

pub fn get_user_id(message: &Message) -> Result<i64, HandleUpdateError> {
    i64::try_from(
        message
            .from
            .as_ref()
            .ok_or_else(|| HandleUpdateError::Command("from in the message is empty".into()))?
            .id,
    )
    .map_err(|e| HandleUpdateError::Command(e.to_string()))
}
