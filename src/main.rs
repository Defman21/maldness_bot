use std::collections::HashMap;
use std::error::Error;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use frankenstein::{Api, ChatId, GetUpdatesParams, LeaveChatParams, TelegramApi, Update};

use crate::cache::Cache;
use crate::commands::{
    donate, gn, rafk, set_my_location, set_paying_status, shuffle, up, weather, work,
};
use crate::errors::HandleUpdateError;
use crate::settings::Settings;
use crate::updates::UpdateHandler;

mod cache;
mod commands;
mod errors;
mod helpers;
mod schema;
mod services;
mod settings;
mod updates;

fn handle_updates(updates: Vec<Update>, handler: &mut UpdateHandler) -> Option<u32> {
    let mut last_update_id: Option<u32> = None;

    let mut chats_to_leave: HashMap<i64, ()> = HashMap::new();

    for update in updates {
        if let Err(err) = handler.handle_update(&update) {
            println!("Error: {}", err);
            if let HandleUpdateError::NotAllowed {
                chat_id, chat_type, ..
            } = err
            {
                if chat_type.as_str() != "private" {
                    chats_to_leave.insert(chat_id, ());
                }
            }
        }
        last_update_id = Some(update.update_id);
    }

    for (chat_id, _) in chats_to_leave.into_iter() {
        if let Err(err) = handler
            .api
            .leave_chat(&LeaveChatParams::new(ChatId::Integer(chat_id)))
            .map(|response| println!("Left: {}", response.result))
            .map_err(HandleUpdateError::from)
        {
            println!("Failed to leave the chat: {}", err);
        }
    }

    last_update_id
}

fn main() -> Result<(), Box<dyn Error>> {
    let settings = Settings::new().unwrap_or_else(|err| {
        println!("Couldn't parse the config! {}", err);
        exit(1);
    });
    let api = Api::new(settings.token.as_str());
    let cache = Cache::new();

    let mut handler = UpdateHandler::new(&api, &settings, &cache);
    handler.commands_executor.register(up::UP);
    handler.commands_executor.register(donate::DONATE);
    handler
        .commands_executor
        .register(set_paying_status::SET_PAYING_STATUS);
    handler.commands_executor.register(weather::WEATHER);
    handler
        .commands_executor
        .register(set_my_location::SET_MY_LOCATION);
    handler.commands_executor.register(gn::GOOD_NIGHT);
    handler.commands_executor.register(shuffle::SHUFFLE);
    handler.commands_executor.register(work::WORK);
    handler.commands_executor.register(rafk::RAFK);
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
