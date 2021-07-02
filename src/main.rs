use std::error::Error;
use std::thread::sleep;
use std::time::Duration;

use frankenstein::{Api, GetUpdatesParams, TelegramApi, Update};

use commands::{CommandExecutor, donate, set_paying_status, up};
use errors::HandleUpdateError;

use crate::commands::weather;
use crate::settings::Settings;

mod commands;
mod errors;
mod services;
mod settings;

const BOT_COMMAND: &str = "bot_command";

fn handle_updates(
    updates: Vec<Update>,
    executor: &mut CommandExecutor,
) -> Result<Option<u32>, HandleUpdateError> {
    let mut last_update_id: Option<u32> = None;

    for update in updates {
        let message = update
            .message
            .as_ref()
            .ok_or(HandleUpdateError::Skip(update.update_id))?;
        let text = message
            .text
            .as_ref()
            .ok_or(HandleUpdateError::Skip(update.update_id))?;

        for entity in message
            .entities
            .as_ref()
            .ok_or(HandleUpdateError::Skip(update.update_id))?
        {
            if entity.type_field.as_str() != BOT_COMMAND {
                continue;
            }

            let text_str = text.as_str();
            let offset = entity.offset as usize;
            let length = entity.length as usize;
            let command = &text_str[offset..length];
            if let Some(err) = executor.execute_command(&update, command, &text_str[length..]) {
                println!("Error: {}", err);
            }
        }

        last_update_id = Some(update.update_id);
    }

    Ok(last_update_id)
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut settings = Settings::new()?;
    let api = Api::new(settings.token.as_str());

    let mut executor = CommandExecutor::new(&api, &mut settings);
    executor.register_command(up::UP);
    executor.register_command(donate::DONATE);
    executor.register_command(set_paying_status::SET_PAYING_STATUS);
    executor.register_command(weather::WEATHER);
    executor.set_commands();

    let mut update_params = GetUpdatesParams::new();
    update_params.set_allowed_updates(Some(vec!["message".to_string()]));

    loop {
        let result = api.get_updates(&update_params);
        match result {
            Ok(response) => match handle_updates(response.result, &mut executor) {
                Ok(Some(last_update_id)) => update_params.set_offset(Some(last_update_id + 1)),
                Ok(None) => {}
                Err(error) => {
                    println!("Error: {:?}", error);

                    match error {
                        HandleUpdateError::Skip(last_update_id) => {
                            update_params.set_offset(Some(last_update_id + 1))
                        }
                        _ => {
                            println!("Unexpected error from the updates handler, could not set the offset");
                        }
                    };
                }
            },
            Err(error) => println!("Error: {:?}", error),
        };
        sleep(Duration::new(3, 0));
    }
}
