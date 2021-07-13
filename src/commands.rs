use crate::cache::Cache;
use crate::errors::HandleUpdateError;
use crate::settings::Settings;
use diesel::PgConnection;
use frankenstein::{
    Api, BotCommand, ChatAction, ChatId, Message, SendChatActionParams, TelegramApi, Update,
};
use std::collections::HashMap;

pub mod donate;
pub mod gn;
pub mod set_my_location;
pub mod set_paying_status;
pub mod shuffle;
pub mod up;
pub mod weather;

pub type CommandResult<T> = Result<(), T>;
pub struct CommandParams<'a> {
    api: &'a Api,
    conn: &'a mut PgConnection,
    cache: &'a Cache,
    settings: &'a Settings,
    message: &'a Message,
    args: &'a str,
}

pub struct Command {
    pub name: &'static str,
    pub description: &'static str,
    // The lifetime of CommandParams is the lifetime of CommandsExecutor.execute (e.g. 'execute)
    // We can't write it like type Handler<'execute> = fn(CommandParams<'execute>) -> ... because
    // we don't know about the 'execute lifetime at that point.
    // So instead we use higher-rank trait bounds: https://doc.rust-lang.org/nomicon/hrtb.html
    pub handler: for<'a> fn(CommandParams<'a>) -> CommandResult<HandleUpdateError>,
    pub is_admin_only: bool,
    pub chat_action: Option<ChatAction>,
}

pub struct CommandsExecutor<'a> {
    settings: &'a Settings,
    tg_api: &'a Api,
    commands: HashMap<String, Command>,
    cache: &'a Cache,
}

impl<'a> CommandsExecutor<'a> {
    pub fn new(settings: &'a Settings, tg_api: &'a Api, cache: &'a Cache) -> Self {
        Self {
            settings,
            tg_api,
            commands: HashMap::new(),
            cache,
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
        self.settings.is_admin(uid)
    }

    pub fn execute(
        &self,
        bot_prefix: &str,
        conn: &mut PgConnection,
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
            if let Some(chat_action) = command.chat_action.as_ref() {
                let _ = self.tg_api.send_chat_action(&SendChatActionParams::new(
                    ChatId::Integer(message.chat.id),
                    chat_action.clone(),
                ));
            }
            if command.is_admin_only && !self.is_admin(update.message.as_ref()?.from.as_ref()?.id) {
                return None;
            }
            return match (command.handler)(CommandParams {
                api: self.tg_api,
                conn,
                cache: self.cache,
                settings: self.settings,
                message,
                args,
            }) {
                Ok(_) => None,
                Err(e) => Some(e),
            };
        }
        None
    }
}
