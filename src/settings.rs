use std::collections::HashMap;
use std::fmt::Debug;

use config::{Config, ConfigError, Value};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OpenWeatherSettings {
    pub api_key: String,
    pub language: String,
    pub units: String,
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

        s.try_into()
    }
}
