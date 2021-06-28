use crate::services::user::errors::ServiceError;
use postgres::Client;

pub struct User {
    id: i32,
    telegram_uid: i64,
    is_paying: bool,
}

pub fn get_by_id(client: &mut Client, telegram_uid: i64) -> Result<User, ServiceError> {
    let row = client
        .query_one(
            "SELECT id, telegram_uid, is_paying FROM users WHERE telegram_uid = $1",
            &[&telegram_uid],
        )
        .or(Err(ServiceError::NotFound(None)))?;

    Ok(User {
        id: row.get(0),
        telegram_uid: row.get(1),
        is_paying: row.get(2),
    })
}

pub fn set_paying_status(
    client: &mut Client,
    telegram_uid: i64,
    is_paying: bool,
) -> Result<User, ServiceError> {
    let result = get_by_id(client, telegram_uid);
    match result {
        Ok(user) => {
            let local_user = User {
                id: user.id,
                telegram_uid: user.telegram_uid,
                is_paying,
            };

            client.execute(
                "UPDATE users SET is_paying = $1 WHERE telegram_uid = $2",
                &[&is_paying, &telegram_uid],
            )?;

            Ok(local_user)
        }
        Err(e) => Err(e),
    }
}
