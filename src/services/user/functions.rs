use crate::services::user::errors::ServiceError;
use diesel::prelude::*;
use diesel::result::Error;

#[derive(Queryable)]
pub struct User {
    id: i32,
    telegram_uid: i64,
    is_paying: bool,
}

#[derive(Insertable)]
#[table_name = "crate::schema::users"]
struct InsertableUser {
    telegram_uid: i64,
    is_paying: bool,
}

pub type Result<T> = std::result::Result<T, ServiceError>;

pub fn get_by_id(client: &mut PgConnection, telegram_uid: i64) -> Result<User> {
    use crate::schema::users::dsl::{telegram_uid as tg_uid, users};

    match users
        .filter(tg_uid.eq(telegram_uid))
        .get_result::<User>(client)
    {
        Ok(user) => Ok(user),
        Err(err) => match err {
            Error::NotFound => Err(ServiceError::NotFound),
            _ => Err(ServiceError::Default(err.to_string())),
        },
    }
}

pub fn create(client: &mut PgConnection, telegram_uid: i64, is_paying: bool) -> Result<User> {
    use crate::schema::users::dsl::{id, users};

    match diesel::insert_into(users)
        .values(InsertableUser {
            telegram_uid,
            is_paying,
        })
        .returning(id)
        .get_result(client)
    {
        Ok(user_id) => Ok(User {
            id: user_id,
            telegram_uid,
            is_paying,
        }),
        Err(e) => Err(ServiceError::Default(e.to_string())),
    }
}

pub fn set_paying_status(
    client: &mut PgConnection,
    telegram_uid: i64,
    is_paying: bool,
) -> Result<User> {
    let result = get_by_id(client, telegram_uid);
    match result {
        Ok(_) => {
            use crate::schema::users::dsl::{
                is_paying as is_paying_db, telegram_uid as tg_uid, users,
            };

            match diesel::update(users.filter(tg_uid.eq(telegram_uid)))
                .set(is_paying_db.eq(is_paying))
                .get_result::<User>(client)
            {
                Ok(user) => Ok(user),
                Err(e) => Err(ServiceError::Default(e.to_string())),
            }
        }
        Err(e) => match e {
            ServiceError::NotFound => create(client, telegram_uid, is_paying),
            _ => Err(e),
        },
    }
}
