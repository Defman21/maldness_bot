use crate::services::user::errors::ServiceError;
use diesel::prelude::*;
use diesel::result::Error;

#[derive(Identifiable, Queryable)]
#[table_name = "crate::schema::users"]
pub struct User {
    pub id: i32,
    telegram_uid: i64,
    is_paying: bool,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    first_name: Option<String>,
    last_name: Option<String>,
    username: Option<String>,
}

#[derive(Insertable)]
#[table_name = "crate::schema::users"]
struct InsertableUser {
    telegram_uid: i64,
    is_paying: Option<bool>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    first_name: Option<String>,
    last_name: Option<String>,
    username: Option<String>,
}

pub type Result<T> = std::result::Result<T, ServiceError>;

pub fn get_by_telegram_user(conn: &mut PgConnection, user: &frankenstein::User) -> Result<User> {
    use crate::schema::users::dsl::{telegram_uid as tg_uid, users};

    match users
        .filter(tg_uid.eq(user.id as i64))
        .get_result::<User>(conn)
    {
        Ok(user) => Ok(user),
        Err(err) => match err {
            Error::NotFound => Err(ServiceError::NotFound),
            _ => Err(err.into()),
        },
    }
}

pub fn get_by_telegram_user_or_create(
    conn: &mut PgConnection,
    user: &frankenstein::User,
) -> Result<User> {
    get_by_telegram_user(conn, user).or_else(|err| match err {
        ServiceError::NotFound => create(conn, user, None, None, None),
        err => Err(err),
    })
}

pub fn create(
    conn: &mut PgConnection,
    user: &frankenstein::User,
    is_paying: Option<bool>,
    latitude: Option<f64>,
    longitude: Option<f64>,
) -> Result<User> {
    use crate::schema::users::dsl::users;

    diesel::insert_into(users)
        .values(InsertableUser {
            telegram_uid: user.id as i64,
            is_paying,
            latitude,
            longitude,
            first_name: Some(user.first_name.clone()),
            last_name: user.last_name.clone(),
            username: user.username.clone(),
        })
        .get_result::<User>(conn)
        .map_err(ServiceError::from)
}

pub fn set_paying_status(
    conn: &mut PgConnection,
    user: &frankenstein::User,
    is_paying: bool,
) -> Result<User> {
    use crate::schema::users::dsl::{
        is_paying as is_paying_db, telegram_uid as telegram_uid_db, users,
    };

    let user = get_by_telegram_user_or_create(conn, user)?;

    diesel::update(users.filter(telegram_uid_db.eq(user.telegram_uid)))
        .set(is_paying_db.eq(is_paying))
        .get_result::<User>(conn)
        .map_err(ServiceError::from)
}

pub fn set_location(
    conn: &mut PgConnection,
    user: &frankenstein::User,
    latitude: f64,
    longitude: f64,
) -> Result<User> {
    use crate::schema::users::dsl::{
        latitude as latitude_db, longitude as longitude_db, telegram_uid as telegram_uid_db, users,
    };

    let user = get_by_telegram_user_or_create(conn, user)?;

    diesel::update(users.filter(telegram_uid_db.eq(user.telegram_uid)))
        .set((latitude_db.eq(latitude), longitude_db.eq(longitude)))
        .get_result::<User>(conn)
        .map_err(ServiceError::from)
}
