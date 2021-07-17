pub mod errors;
pub mod functions;

use frankenstein::User;
use humantime::format_duration;
use liquid::Template;
use serde::Serialize;
use std::time::Duration;

#[derive(Debug, Serialize)]
struct AfkEventTemplateGlobals {
    username: String,
    message: String,
    duration: String,
}

fn get_username(from: &User) -> String {
    match &from.username {
        Some(username) => username.clone(),
        None => vec![
            from.first_name.clone(),
            from.last_name.clone().unwrap_or_else(|| "".into()),
        ]
        .join(" "),
    }
}

pub fn render_template(
    template: &Template,
    from: &User,
    message: Option<&String>,
    duration: Duration,
) -> String {
    let globals = liquid::to_object(&AfkEventTemplateGlobals {
        username: get_username(from),
        message: message.cloned().unwrap_or_else(|| "N/A".into()),
        duration: format_duration(duration).to_string(),
    })
    .expect("Failed to serialize AfkEventTemplateGlobals to liquid::Object");

    template
        .render(&globals)
        .expect("Failed to render a template")
}
