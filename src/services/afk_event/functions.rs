use crate::services::afk_event::errors::ServiceError;
use crate::services::user::functions::{
    get_by_telegram_user, get_by_telegram_user_or_create, User,
};

use crate::services::afk_event::render_template;
use crate::settings::Settings;
use chrono::prelude::*;
use diesel::prelude::*;
use diesel::result::Error;
use frankenstein::Message;
use std::time::Duration;

#[derive(Copy, Clone)]
pub enum EventType {
    Sleep = 1,
    Work = 2,
}

impl From<i32> for EventType {
    fn from(v: i32) -> Self {
        match v {
            1 => EventType::Sleep,
            2 => EventType::Work,
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
        let event_duration = Duration::from_secs(
            (self.ended_at.unwrap() - self.started_at)
                .to_std()
                .unwrap()
                .as_secs(),
        );

        let template = match EventType::from(self.event_type) {
            EventType::Sleep => settings.wake_up_template(),
            EventType::Work => settings.back_from_work_template(),
        };

        render_template(
            template,
            message.from.as_ref().unwrap(),
            self.message.as_ref(),
            event_duration,
        )
    }

    pub fn event_type(&self) -> EventType {
        EventType::from(self.event_type)
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
    user: &frankenstein::User,
    event_type: EventType,
    action_type: ActionType,
    message: Option<String>,
) -> Result<AfkEvent> {
    let user = get_by_telegram_user_or_create(conn, user)?;

    match action_type {
        ActionType::Continue => match get_latest_event(conn, &user) {
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

pub fn reset_latest_event(conn: &mut PgConnection, user: &frankenstein::User) -> Result<AfkEvent> {
    let user = get_by_telegram_user(conn, user)?;
    let event = get_latest_event(conn, &user)?;

    reset_event(conn, event.id)
}

fn get_latest_event(conn: &mut PgConnection, user: &User) -> Result<AfkEvent> {
    use crate::schema::afk_events::dsl::id;

    AfkEvent::belonging_to(user)
        .order_by(id.desc())
        .get_result::<AfkEvent>(conn)
        .map_err(|err| match err {
            Error::NotFound => ServiceError::NotFound,
            err => ServiceError::Default(err.to_string()),
        })
}

pub fn get_afk_users(conn: &mut PgConnection) -> Result<Vec<(i64, i32)>> {
    use crate::schema::{
        afk_events::dsl::{afk_events, ended_at, id},
        users::dsl::{telegram_uid, users},
    };

    users
        .inner_join(afk_events)
        .select((telegram_uid, id))
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
