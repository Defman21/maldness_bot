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
}

#[derive(Insertable)]
#[table_name = "crate::schema::users"]
struct InsertableUser {
    telegram_uid: i64,
    is_paying: Option<bool>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

pub type Result<T> = std::result::Result<T, ServiceError>;

pub fn get_by_telegram_uid(conn: &mut PgConnection, telegram_uid: i64) -> Result<User> {
    use crate::schema::users::dsl::{telegram_uid as tg_uid, users};

    match users
        .filter(tg_uid.eq(telegram_uid))
        .get_result::<User>(conn)
    {
        Ok(user) => Ok(user),
        Err(err) => match err {
            Error::NotFound => Err(ServiceError::NotFound),
            _ => Err(err.into()),
        },
    }
}

pub fn get_by_telegram_uid_or_create(conn: &mut PgConnection, telegram_uid: i64) -> Result<User> {
    get_by_telegram_uid(conn, telegram_uid).or_else(|err| match err {
        ServiceError::NotFound => create(conn, telegram_uid, None, None, None),
        err => Err(err),
    })
}

pub fn create(
    conn: &mut PgConnection,
    telegram_uid: i64,
    is_paying: Option<bool>,
    latitude: Option<f64>,
    longitude: Option<f64>,
) -> Result<User> {
    use crate::schema::users::dsl::{id, users};

    match diesel::insert_into(users)
        .values(InsertableUser {
            telegram_uid,
            is_paying,
            latitude,
            longitude,
        })
        .returning(id)
        .get_result(conn)
    {
        Ok(user_id) => Ok(User {
            id: user_id,
            telegram_uid,
            is_paying: is_paying.unwrap_or(false),
            latitude,
            longitude,
        }),
        Err(err) => Err(err.into()),
    }
}

pub fn set_paying_status(
    conn: &mut PgConnection,
    telegram_uid: i64,
    is_paying: bool,
) -> Result<User> {
    match get_by_telegram_uid(conn, telegram_uid) {
        Ok(_) => {
            use crate::schema::users::dsl::{
                is_paying as is_paying_db, telegram_uid as telegram_uid_db, users,
            };

            diesel::update(users.filter(telegram_uid_db.eq(telegram_uid)))
                .set(is_paying_db.eq(is_paying))
                .get_result::<User>(conn)
                .map_err(ServiceError::from)
        }
        Err(err) => match err {
            ServiceError::NotFound => create(conn, telegram_uid, Some(is_paying), None, None),
            _ => Err(err),
        },
    }
}

pub fn set_location(
    conn: &mut PgConnection,
    telegram_uid: i64,
    latitude: f64,
    longitude: f64,
) -> Result<User> {
    match get_by_telegram_uid(conn, telegram_uid) {
        Ok(_) => {
            use crate::schema::users::dsl::{
                latitude as latitude_db, longitude as longitude_db,
                telegram_uid as telegram_uid_db, users,
            };

            diesel::update(users.filter(telegram_uid_db.eq(telegram_uid)))
                .set((latitude_db.eq(latitude), longitude_db.eq(longitude)))
                .get_result::<User>(conn)
                .map_err(ServiceError::from)
        }
        Err(err) => match err {
            ServiceError::NotFound => {
                create(conn, telegram_uid, None, Some(latitude), Some(longitude))
            }
            _ => Err(err),
        },
    }
}
