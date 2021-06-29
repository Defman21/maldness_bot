use frankenstein::{Api, BotCommand, SetMyCommandsParams, TelegramApi, Update};
use std::collections::HashMap;

use crate::errors::HandleUpdateError;
use crate::settings::Settings;
use postgres::{Client, NoTls};

pub mod donate;
pub mod set_paying_status;
pub mod up;

type Handler = fn(&Api, &Update, &mut Client, &Settings, &str) -> Option<HandleUpdateError>;

pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub handler: Handler,
    pub is_admin_only: bool,
}

pub struct CommandExecutor<'a> {
    api: &'a Api,
    settings: &'a Settings,
    commands: HashMap<String, Command>,
    bot_prefix: String,
    postgres: Client,
}

impl<'a> CommandExecutor<'a> {
    pub fn new(api: &'a Api, settings: &'a mut Settings) -> CommandExecutor<'a> {
        let mut executor = CommandExecutor {
            api,
            settings,
            commands: HashMap::new(),
            bot_prefix: String::new(),
            postgres: Client::connect(settings.postgres_dsn.as_str(), NoTls)
                .expect("Failed to connect to postgres"),
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

    fn is_admin(&self, user_id: u64) -> bool {
        self.settings.admins.contains(&user_id)
    }

    pub fn set_bot_prefix(&mut self, prefix: String) {
        self.bot_prefix = prefix;
    }

    pub fn set_commands(&self) {
        let commands: Vec<BotCommand> = self
            .commands
            .values()
            .map(|cmd| {
                let mut description = String::new();
                if cmd.is_admin_only {
                    description += "[Admin only] ";
                }
                description += cmd.description;

                BotCommand::new(cmd.name.to_string(), description)
            })
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

    pub fn execute_command(
        &mut self,
        update: &Update,
        command_entity: &str,
        args: &str,
    ) -> Option<HandleUpdateError> {
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
                return None;
            }
        }

        println!("Command name: {:?}", command_name);

        if let Some(command) = self.commands.get(command_name) {
            if command.is_admin_only && !self.is_admin(update.message.as_ref()?.from.as_ref()?.id) {
                return None;
            }
            return (command.handler)(self.api, update, &mut self.postgres, self.settings, args);
        }
        None
    }
}
