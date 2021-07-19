use config::{Config, ConfigError};
use frankenstein::Message;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use crate::commands::{donate, gn, shuffle, weather, work};
use crate::errors::HandleUpdateError;

#[derive(Deserialize)]
pub struct OpenWeatherSettings {
    pub api_key: String,
    pub language: String,
    pub units: String,
    message_format: Option<String>,
    #[serde(skip)]
    _message_format_tpl: Option<liquid::Template>,
}

impl Debug for OpenWeatherSettings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "OpenWeatherSettings<api_key={}, language={}, units={}, message_format={:?} (liquid::Template initialized: {})>",
               self.api_key, self.language, self.units, self.message_format, self._message_format_tpl.is_some())
    }
}

impl OpenWeatherSettings {
    pub fn message_format(&self) -> &liquid::Template {
        self._message_format_tpl.as_ref().unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub struct CommandsMap {
    pub donate: donate::CommandSettings,
    pub gn: gn::CommandSettings,
    pub weather: weather::CommandSettings,
    pub shuffle: shuffle::CommandSettings,
    pub work: work::CommandSettings,
}

#[derive(Debug, Deserialize)]
struct AllowedChatsSettings {
    allow_unspecified: Option<bool>,
    private: Option<HashMap<i64, bool>>,
    group: Option<HashMap<i64, bool>>,
    supergroup: Option<HashMap<i64, bool>>,
}

#[derive(Deserialize)]
pub struct Settings {
    pub token: String,
    pub postgres_dsn: String,
    pub admins: Vec<u64>,
    #[serde(skip)]
    _admins_map: Option<HashMap<u64, ()>>,
    pub commands: CommandsMap,
    pub open_weather: OpenWeatherSettings,
    wake_up_format: Option<String>,
    back_from_work_format: Option<String>,
    #[serde(skip)]
    _wake_up_format_tpl: Option<liquid::Template>,
    #[serde(skip)]
    _back_from_work_format_tpl: Option<liquid::Template>,
    allowed_chats: AllowedChatsSettings,
}

impl Debug for Settings {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<Settings token={} postgres_dsn={} admins={:?} commands={:?} open_weather={:?} \
        wake_up_format={:?} back_from_work_format={:?} allowed_chats={:?}>",
            self.token,
            self.postgres_dsn,
            self.admins,
            self.commands,
            self.open_weather,
            self.wake_up_format,
            self.back_from_work_format,
            self.allowed_chats
        )
    }
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::default();
        s.merge(config::File::with_name("config"))?
            .merge(config::Environment::with_prefix("bot"))?;

        let mut s: Self = s.try_into()?;

        s._admins_map = Some(s.admins.iter().map(|i| (i.to_owned(), ())).collect());

        if s.open_weather.message_format.is_none() {
            s.open_weather.message_format = Some(
                "{{ name }}: {{ temp }} (feels like {{ feels_like }}), {{ description }}".into(),
            );
        }

        if s.wake_up_format.is_none() {
            s.wake_up_format = Some(
                "{{ username }} have finished their sleep: {{ message }}. They've slept for {{ duration }}"
                    .into()
            );
        }

        if s.back_from_work_format.is_none() {
            s.back_from_work_format = Some(
                "{{ username }} have finished working: {{ message }}. They've worked for {{ duration }}"
                    .into(),
            );
        }

        s.open_weather._message_format_tpl = Some(
            liquid::ParserBuilder::with_stdlib()
                .build()
                .map_err(|e| ConfigError::Message(e.to_string()))?
                .parse(s.open_weather.message_format.as_ref().unwrap().as_str())
                .map_err(|e| {
                    println!("There's an error in your [open_weather].message_format setting!");
                    ConfigError::Message(e.to_string())
                })?,
        );

        s._wake_up_format_tpl = Some(
            liquid::ParserBuilder::with_stdlib()
                .build()
                .map_err(|e| ConfigError::Message(e.to_string()))?
                .parse(s.wake_up_format.as_ref().unwrap().as_str())
                .map_err(|e| {
                    println!("There's an error in your wake_up_format setting!");
                    ConfigError::Message(e.to_string())
                })?,
        );

        s._back_from_work_format_tpl = Some(
            liquid::ParserBuilder::with_stdlib()
                .build()
                .map_err(|e| ConfigError::Message(e.to_string()))?
                .parse(s.back_from_work_format.as_ref().unwrap().as_str())
                .map_err(|e| {
                    println!("There's an error in your back_from_work_format setting!");
                    ConfigError::Message(e.to_string())
                })?,
        );

        Ok(s)
    }

    pub fn is_admin(&self, user_id: u64) -> bool {
        self._admins_map.as_ref().unwrap().contains_key(&user_id)
    }

    pub fn check_for_allowed_update(
        &self,
        Message { chat, from, .. }: &Message,
    ) -> Option<HandleUpdateError> {
        let chat_type = chat.type_field.as_str();

        let formatted_chat_title = || {
            let res;

            if let Some(title) = chat.title.clone() {
                if let Some(username) = chat.username.clone() {
                    res = format!("title={} username={}", title, username);
                } else {
                    res = format!("title={}", title);
                }
            } else {
                res = format!(
                    "Chat with first_name={:?} last_name={:?} username={:?}",
                    chat.first_name, chat.last_name, chat.username
                );
            }

            res
        };

        let chat_id = chat.id;

        if chat_type == "channel" {
            return Some(HandleUpdateError::NotAllowed {
                chat_id,
                reason: "The bot does not support channels".into(),
                chat_name: formatted_chat_title(),
                chat_type: chat_type.to_string(),
            });
        }

        let from_id = match from.as_ref() {
            Some(from) => from.id,
            None => 0,
        };

        if self.is_admin(from_id) {
            return None;
        }

        let default = self.allowed_chats.allow_unspecified.unwrap_or(true);

        let chat_id_allowed_map = match chat_type {
            "private" => self.allowed_chats.private.as_ref(),
            "group" => self.allowed_chats.group.as_ref(),
            "supergroup" => self.allowed_chats.supergroup.as_ref(),
            _ => panic!("undefined chat_type: {}", chat_type),
        };

        let mut reason = String::new();

        let allowed = chat_id_allowed_map
            .and_then(|map| {
                if map.get(&0).is_some() {
                    return Some(&true);
                }

                reason = format!(
                    "configuration: [allowed_chats].{}[{}] is false",
                    chat_type, chat_id
                );
                map.get(&chat_id)
            })
            .or_else(|| {
                reason = String::from("configuration: [allowed_chats].allow_unspecified is false");
                Some(&default)
            })
            .unwrap()
            .to_owned();

        if !allowed {
            Some(HandleUpdateError::NotAllowed {
                chat_id,
                reason,
                chat_name: formatted_chat_title(),
                chat_type: chat_type.to_string(),
            })
        } else {
            None
        }
    }

    pub fn wake_up_template(&self) -> &liquid::Template {
        self._wake_up_format_tpl.as_ref().unwrap()
    }

    pub fn back_from_work_template(&self) -> &liquid::Template {
        self._back_from_work_format_tpl.as_ref().unwrap()
    }
}
