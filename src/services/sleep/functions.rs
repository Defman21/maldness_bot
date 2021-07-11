use crate::services::sleep::errors::ServiceError;
use crate::services::user::{functions::{User, get_by_telegram_uid, create as create_user}, errors::ServiceError as UserServiceError};

use chrono::prelude::*;
use diesel::prelude::*;
use diesel::result::Error;

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(User)]
#[table_name = "crate::schema::sleep_events"]
pub struct SleepEvent {
    pub id: i32,
    pub started_at: chrono::NaiveDateTime,
    pub ended_at: Option<chrono::NaiveDateTime>,
    pub message: Option<String>,
    pub user_id: i32,
}

#[derive(Insertable)]
#[table_name = "crate::schema::sleep_events"]
struct InsertableSleepEvent {
    started_at: chrono::NaiveDateTime,
    ended_at: Option<Option<chrono::NaiveDateTime>>,
    message: Option<Option<String>>,
    user_id: i32,
}

pub type Result<T> = std::result::Result<T, ServiceError>;

pub enum SleepType {
    Continue,
    New,
}

pub fn go_to_sleep(
    user_id: i64,
    sleep_type: SleepType,
    message: Option<String>,
    conn: &mut PgConnection,
) -> Result<SleepEvent> {
    let user = match get_by_telegram_uid(conn, user_id) {
        Ok(user) => user,
        Err(UserServiceError::NotFound) => create_user(conn, user_id, None, None, None)?,
        Err(err) => return Err(ServiceError::from(err)),
    };
    match sleep_type {
        SleepType::Continue => match get_last_not_ended_event(&user, conn) {
            Ok(event) => {
                use crate::schema::sleep_events::dsl::{ended_at, id as id_db, sleep_events};
                diesel::update(sleep_events.filter(id_db.eq(event.id)))
                    .set(ended_at.eq::<Option<NaiveDateTime>>(None))
                    .get_result::<SleepEvent>(conn)
                    .map_err(ServiceError::from)
            }
            Err(ServiceError::NotFound) => create(conn, user.id, message),
            Err(err) => Err(err),
        },
        SleepType::New => create(conn, user.id, message),
    }
}

pub fn end_sleep(user_id: i64, conn: &mut PgConnection) -> Result<SleepEvent> {
    let user = crate::services::user::functions::get_by_telegram_uid(conn, user_id)?;
    let event = get_last_not_ended_event(&user, conn)?;

    use crate::schema::sleep_events::dsl::{ended_at, id, sleep_events};
    diesel::update(sleep_events.filter(id.eq(event.id)))
        .set(ended_at.eq(Some(Local::now().naive_utc())))
        .get_result(conn)
        .map_err(ServiceError::from)
}

fn get_last_not_ended_event(user: &User, conn: &mut PgConnection) -> Result<SleepEvent> {
    use crate::schema::sleep_events::dsl::id;

    SleepEvent::belonging_to(user)
        .order_by(id.desc())
        .get_result::<SleepEvent>(conn)
        .map_err(|err| match err {
            Error::NotFound => ServiceError::NotFound,
            err => ServiceError::Default(err.to_string()),
        })
}

pub fn get_sleeping_users(conn: &mut PgConnection) -> Result<Vec<i64>> {
    use crate::schema::{
        sleep_events::dsl::{ended_at, sleep_events, user_id},
        users::dsl::{id, telegram_uid, users},
    };

    users
        .select(telegram_uid)
        .filter(id.eq_any(sleep_events.select(user_id).filter(ended_at.is_null())))
        .get_results::<i64>(conn)
        .map_err(ServiceError::from)
}

fn create(conn: &mut PgConnection, user_id: i32, message: Option<String>) -> Result<SleepEvent> {
    use crate::schema::sleep_events::dsl::{id, sleep_events};

    let started_at = Local::now().naive_utc();

    match diesel::insert_into(sleep_events)
        .values(InsertableSleepEvent {
            started_at,
            ended_at: None,
            message: Some(message.clone()),
            user_id,
        })
        .returning(id)
        .get_result(conn)
    {
        Ok(event_id) => Ok(SleepEvent {
            id: event_id,
            started_at,
            ended_at: None,
            message,
            user_id,
        }),
        Err(err) => Err(err.into()),
    }
}
