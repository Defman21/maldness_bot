use crate::errors::HandleUpdateError;
use crate::settings::Settings;
use diesel::PgConnection;
use frankenstein::{Api, BotCommand, Update, Message};
use std::collections::HashMap;

pub mod donate;
pub mod set_my_location;
pub mod set_paying_status;
pub mod up;
pub mod weather;

pub type CommandResult<T> = Result<(), T>;
type Handler =
    fn(&Api, &Update, &mut PgConnection, &Settings, &Message, &str) -> CommandResult<HandleUpdateError>;

pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    pub handler: Handler,
    pub is_admin_only: bool,
}

pub struct CommandsExecutor<'a> {
    settings: &'a Settings,
    tg_api: &'a Api,
    commands: HashMap<String, Command>,
}

impl<'a> CommandsExecutor<'a> {
    pub fn new(settings: &'a Settings, tg_api: &'a Api) -> Self {
        Self {
            settings,
            tg_api,
            commands: HashMap::new(),
        }
    }

    pub fn bot_commands(&self) -> Vec<BotCommand> {
        self.commands
            .values()
            .map(|cmd| {
                let mut description = String::new();
                if cmd.is_admin_only {
                    description += "[Admin only] ";
                }
                description += cmd.description;

                BotCommand::new(cmd.name.to_string(), description)
            })
            .collect()
    }

    pub fn register(&mut self, command: Command) {
        if self.commands.contains_key(command.name) {
            return;
        }

        self.commands.insert(command.name.to_string(), command);
    }

    fn is_admin(&self, uid: u64) -> bool {
        self.settings.admins.contains(&uid)
    }

    pub fn execute(
        &self,
        bot_prefix: &str,
        postgres: &mut PgConnection,
        update: &Update,
        command_entity: &str,
        message: &Message,
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

            if bot_name != bot_prefix {
                println!(
                    "My name is {}, but the command was called for {}",
                    bot_prefix, bot_name
                );
                return None;
            }
        }

        println!("Command name: {:?}", command_name);

        if let Some(command) = self.commands.get(command_name) {
            if command.is_admin_only && !self.is_admin(update.message.as_ref()?.from.as_ref()?.id) {
                return None;
            }
            return match (command.handler)(self.tg_api, update, postgres, self.settings, message, args) {
                Ok(_) => None,
                Err(e) => Some(e),
            };
        }
        None
    }
}
