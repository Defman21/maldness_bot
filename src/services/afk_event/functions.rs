use crate::services::afk_event::errors::ServiceError;
use crate::services::user::{
    functions::{get_by_telegram_uid_or_create, User},
};

use crate::services::afk_event::format_sleep_data;
use crate::settings::Settings;
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::result::Error;
use frankenstein::Message;
use std::time::Duration;

#[derive(Copy, Clone)]
pub enum EventType {
    Sleep = 1,
}

impl From<i32> for EventType {
    fn from(v: i32) -> Self {
        match v {
            1 => EventType::Sleep,
            _ => panic!("unsupported event type: {}", v),
        }
    }
}

#[derive(Identifiable, Queryable, Associations)]
#[belongs_to(User)]
#[table_name = "crate::schema::afk_events"]
pub struct AfkEvent {
    pub id: i32,
    pub started_at: chrono::NaiveDateTime,
    pub ended_at: Option<chrono::NaiveDateTime>,
    pub message: Option<String>,
    pub user_id: i32,
    pub event_type: i32,
}

impl AfkEvent {
    pub fn to_string(&self, settings: &Settings, message: &Message) -> String {
        let sleep_duration = Duration::from_secs(
            (self.ended_at.unwrap() - self.started_at)
                .to_std()
                .unwrap()
                .as_secs(),
        );

        match EventType::from(self.event_type) {
            EventType::Sleep => format_sleep_data(
                settings,
                message.from.as_ref().unwrap(),
                self.message.as_ref(),
                sleep_duration,
            ),
        }
    }
}

#[derive(Insertable)]
#[table_name = "crate::schema::afk_events"]
struct InsertableAfkEvent {
    started_at: chrono::NaiveDateTime,
    ended_at: Option<Option<chrono::NaiveDateTime>>,
    message: Option<Option<String>>,
    user_id: i32,
    event_type: i32,
}

pub type Result<T> = std::result::Result<T, ServiceError>;

pub enum ActionType {
    Continue,
    New,
}

fn reset_event(conn: &mut PgConnection, event_id: i32) -> Result<AfkEvent> {
    use crate::schema::afk_events::dsl::{afk_events, ended_at, id};

    diesel::update(afk_events.filter(id.eq(event_id)))
        .set(ended_at.eq::<Option<NaiveDateTime>>(None))
        .get_result::<AfkEvent>(conn)
        .map_err(ServiceError::from)
}

pub fn begin_event(
    conn: &mut PgConnection,
    user_id: i64,
    event_type: EventType,
    action_type: ActionType,
    message: Option<String>,
) -> Result<AfkEvent> {
    let user = get_by_telegram_uid_or_create(conn, user_id)?;

    match action_type {
        ActionType::Continue => match get_last_not_ended_event(conn, &user, event_type) {
            Ok(event) => reset_event(conn, event.id),
            Err(ServiceError::NotFound) => create_event(conn, user.id, event_type, message),
            Err(err) => Err(err),
        },
        ActionType::New => create_event(conn, user.id, event_type, message),
    }
}

pub fn end_event(conn: &mut PgConnection, event_id: i32) -> Result<AfkEvent> {
    use crate::schema::afk_events::dsl::{afk_events, ended_at, id};

    diesel::update(afk_events.filter(id.eq(event_id)))
        .set(ended_at.eq(Some(Local::now().naive_utc())))
        .get_result(conn)
        .map_err(ServiceError::from)
}

fn get_last_not_ended_event(
    conn: &mut PgConnection,
    user: &User,
    event_type: EventType,
) -> Result<AfkEvent> {
    use crate::schema::afk_events::dsl::{event_type as event_type_db, id};

    AfkEvent::belonging_to(user)
        .filter(event_type_db.eq(event_type as i32))
        .order_by(id.desc())
        .get_result::<AfkEvent>(conn)
        .map_err(|err| match err {
            Error::NotFound => ServiceError::NotFound,
            err => ServiceError::Default(err.to_string()),
        })
}

pub fn get_afk_users(conn: &mut PgConnection) -> Result<Vec<(i64, i32)>> {
    use crate::schema::{
        afk_events::dsl::{afk_events, ended_at, event_type},
        users::dsl::{telegram_uid, users},
    };

    users
        .inner_join(afk_events)
        .select((telegram_uid, event_type))
        .filter(ended_at.is_null())
        .get_results(conn)
        .map_err(ServiceError::from)
}

fn create_event(
    conn: &mut PgConnection,
    user_id: i32,
    event_type: EventType,
    message: Option<String>,
) -> Result<AfkEvent> {
    use crate::schema::afk_events::dsl::{afk_events, id};

    let started_at = Local::now().naive_utc();

    match diesel::insert_into(afk_events)
        .values(InsertableAfkEvent {
            started_at,
            ended_at: None,
            message: Some(message.clone()),
            user_id,
            event_type: event_type as i32,
        })
        .returning(id)
        .get_result(conn)
    {
        Ok(event_id) => Ok(AfkEvent {
            id: event_id,
            started_at,
            ended_at: None,
            message,
            user_id,
            event_type: event_type as i32,
        }),
        Err(err) => Err(err.into()),
    }
}
