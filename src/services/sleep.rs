pub mod errors;
pub mod functions;

use crate::settings::Settings;
use frankenstein::User;
use humantime::format_duration;
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Serialize)]
struct FormatSleepGlobals {
    username: String,
    message: String,
    duration: String,
}

pub fn format_sleep_data(
    settings: &Settings,
    from: &User,
    message: Option<String>,
    duration: Duration,
) -> String {
    let username = match &from.username {
        Some(username) => username.clone(),
        None => vec![
            from.first_name.clone(),
            from.last_name.clone().unwrap_or_else(|| "".into()),
        ]
        .join(" "),
    };

    let globals = liquid::to_object(&FormatSleepGlobals {
        username,
        message: message.unwrap_or_else(|| "N/A".into()),
        duration: format_duration(duration).to_string(),
    })
    .expect("Failed to serialize FormatSleepGlobals to liquid::Object");

    settings
        .wake_up_format()
        .render(&globals)
        .expect("Failed to render a template")
}
