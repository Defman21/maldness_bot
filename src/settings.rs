use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use config::{Config, ConfigError, Value};
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
pub struct Settings {
    pub token: String,
    pub postgres_dsn: String,
    pub admins: Vec<u64>,
    // TODO: should not be public, we can't take the value out of the command hashmap thus leading
    // to clone calls every time when accessing e.g. commands["donate"]["some_option"]
    pub commands: HashMap<String, HashMap<String, Value>>,
    pub open_weather: OpenWeatherSettings,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::default();
        s.merge(config::File::with_name("config"))?
            .merge(config::Environment::with_prefix("bot"))?;

        let mut s = s.try_into::<Self>().map_err(|e| ConfigError::Message(e.to_string()))?;

        if s.open_weather.message_format.is_none() {
            s.open_weather.message_format = Some(
                "{{ name }}: {{ temp }} (feels like {{ feels_like }}), {{ description }}".into()
            );
        }

        s.open_weather._message_format_tpl =
            Some(liquid::ParserBuilder::with_stdlib()
                .build()
                .map_err(|e| ConfigError::Message(e.to_string()))?
                .parse(s.open_weather.message_format.as_ref().unwrap().as_str())
                .map_err(|e| {
                    println!("There's an error in your [open_weather].message_format string!");
                    ConfigError::Message(e.to_string())
                })?
            );

        Ok(s)
    }
}
