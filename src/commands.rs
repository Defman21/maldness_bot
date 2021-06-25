use frankenstein::{Api, TelegramApi, Update};
use std::collections::HashMap;

use crate::errors::HandleUpdateError;

pub mod up;

type Handler = fn(&Api, &Update, &str) -> Option<HandleUpdateError>;

pub struct CommandExecutor<'a> {
    api: &'a Api,
    commands: HashMap<String, Handler>,
    bot_prefix: String,
}

impl<'a> CommandExecutor<'a> {
    pub fn new(api: &'a Api) -> CommandExecutor<'a> {
        let mut executor = CommandExecutor {
            api,
            commands: HashMap::new(),
            bot_prefix: String::new(),
        };

        match api.get_me() {
            Ok(response) => {
                let prefix = response
                    .result
                    .username
                    .expect("Failed to get the username");
                executor.set_bot_prefix(prefix);
            }
            Err(error) => {
                panic!("Failed to get myself: {:?}", error);
            }
        };

        executor
    }
    pub fn set_bot_prefix(&mut self, prefix: String) {
        self.bot_prefix = prefix;
    }

    pub fn register_command(&mut self, name: String, executor: Handler) {
        if self.commands.contains_key(&*name) {
            return;
        }

        self.commands.insert(name, executor);
    }

    pub fn execute_command(&self, update: &Update, command_entity: &str, args: &str) {
        let mut args = args;
        if !args.is_empty() {
            args = &args[1..];
        }

        let mut command_name = &command_entity[1..];
        if let Some(at_index) = command_name.find('@') {
            let bot_name = &command_entity[at_index + 2..]; // for /a@b at_index = 1, @b is at index 2, and b is at index 3
            command_name = &command_name[..at_index];

            if bot_name != self.bot_prefix.as_str() {
                println!(
                    "My name is {}, but the command was called for {}",
                    self.bot_prefix, bot_name
                );
                return;
            }
        }

        println!("Command name: {:?}", command_name);

        if let Some(handler) = self.commands.get(command_name) {
            if let Some(error) = handler(self.api, update, args) {
                println!("{}", error);
            }
        }
    }
}
