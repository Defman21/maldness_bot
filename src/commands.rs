use frankenstein::{Api, BotCommand, SetMyCommandsParams, TelegramApi, Update};
use std::collections::HashMap;

use crate::errors::HandleUpdateError;

pub mod donate;
pub mod up;

type Handler = fn(&Api, &Update, &str) -> Option<HandleUpdateError>;

pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub handler: Handler,
}

pub struct CommandExecutor<'a> {
    api: &'a Api,
    commands: HashMap<String, Command>,
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

    pub fn set_commands(&self) {
        let commands: Vec<BotCommand> = self
            .commands
            .values()
            .map(|cmd| BotCommand::new(cmd.name.to_string(), cmd.description.to_string()))
            .collect();
        let set_my_commands_params = SetMyCommandsParams::new(commands);
        self.api
            .set_my_commands(&set_my_commands_params)
            .expect("Failed to set my commands");
    }

    pub fn register_command(&mut self, command: Command) {
        if self.commands.contains_key(command.name) {
            return;
        }

        self.commands.insert(command.name.to_string(), command);
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

        if let Some(command) = self.commands.get(command_name) {
            if let Some(error) = (command.handler)(self.api, update, args) {
                println!("{}", error);
            }
        }
    }
}
