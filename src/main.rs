use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

use frankenstein::{Api, GetUpdatesParams, TelegramApi, Update};

use commands::{donate, set_paying_status, up};
use updates::UpdateHandler;

use crate::commands::weather;
use crate::settings::Settings;

mod commands;
mod errors;
mod schema;
mod services;
mod settings;
mod updates;

fn handle_updates(updates: Vec<Update>, executor: &mut UpdateHandler) -> Option<u32> {
    let mut last_update_id: Option<u32> = None;

    for update in updates {
        if let Err(err) = executor.handle_update(&update) {
            println!("Error: {}", err);
        }
        last_update_id = Some(update.update_id);
    }

    last_update_id
}

fn main() -> Result<(), Box<dyn Error>> {
    let settings = Settings::new()?;
    let api = Api::new(settings.token.as_str());

    let mut handler = UpdateHandler::new(&api, &settings);
    handler.commands_executor.register(up::UP);
    handler.commands_executor.register(donate::DONATE);
    handler
        .commands_executor
        .register(set_paying_status::SET_PAYING_STATUS);
    handler.commands_executor.register(weather::WEATHER);
    handler.send_my_commands();

    let mut update_params = GetUpdatesParams::new();
    update_params.set_allowed_updates(Some(vec!["message".to_string()]));

    loop {
        let result = api.get_updates(&update_params);
        match result {
            Ok(response) => {
                if let Some(last_update_id) = handle_updates(response.result, &mut handler) {
                    update_params.set_offset(Some(last_update_id + 1))
                }
            }
            Err(error) => println!("Error: {:?}", error),
        };
        sleep(Duration::new(3, 0));
    }
}
