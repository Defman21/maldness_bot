use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use crate::commands::donate;
use config::{Config, ConfigError};
use serde::Deserialize;

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
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub token: String,
    pub postgres_dsn: String,
    pub admins: Vec<u64>,
    pub commands: CommandsMap,
    pub open_weather: OpenWeatherSettings,
    allowed_chats: Option<Vec<i64>>,
    #[serde(skip)]
    _allowed_chats_hashmap: Option<HashMap<i64, ()>>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::default();
        s.merge(config::File::with_name("config"))?
            .merge(config::Environment::with_prefix("bot"))?;

        let mut s: Self = s.try_into()?;

        if s.open_weather.message_format.is_none() {
            s.open_weather.message_format = Some(
                "{{ name }}: {{ temp }} (feels like {{ feels_like }}), {{ description }}".into(),
            );
        }

        s.open_weather._message_format_tpl = Some(
            liquid::ParserBuilder::with_stdlib()
                .build()
                .map_err(|e| ConfigError::Message(e.to_string()))?
                .parse(s.open_weather.message_format.as_ref().unwrap().as_str())
                .map_err(|e| {
                    println!("There's an error in your [open_weather].message_format string!");
                    ConfigError::Message(e.to_string())
                })?,
        );

        if let Some(allowed_chats) = s.allowed_chats.as_ref() {
            s._allowed_chats_hashmap =
                Some(allowed_chats.iter().map(|i| (i.to_owned(), ())).collect());
        };

        Ok(s)
    }

    pub fn is_chat_allowed(&self, chat_id: i64) -> bool {
        if let Some(allowed_chats_map) = self._allowed_chats_hashmap.as_ref() {
            allowed_chats_map.contains_key(&chat_id)
        } else {
            true
        }
    }
}
