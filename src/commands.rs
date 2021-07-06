use std::collections::HashMap;

use frankenstein::{
    Api, BotCommand, ChatId, Location, Message, MessageEntity, SendMessageParams,
    SetMyCommandsParams, TelegramApi, Update,
};
use postgres::{Client, NoTls};

use crate::errors::HandleUpdateError;
use crate::services::weather::{format_weather_data, get_weather, Identifier};
use crate::settings::Settings;

pub mod donate;
pub mod set_paying_status;
pub mod up;
pub mod weather;

pub type CommandResult<T> = Result<(), T>;
type Handler = fn(&Api, &Update, &mut Client, &Settings, &str) -> CommandResult<HandleUpdateError>;

const BOT_COMMAND: &str = "bot_command";

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

    pub fn send_my_commands(&self) {
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
            return match (command.handler)(
                self.api,
                update,
                &mut self.postgres,
                self.settings,
                args,
            ) {
                Ok(_) => None,
                Err(e) => Some(e),
            };
        }
        None
    }

    fn handle_command(
        &mut self,
        update: &Update,
        command_entity: &MessageEntity,
        text: &str,
    ) -> Result<(), HandleUpdateError> {
        let offset = command_entity.offset as usize;
        let length = command_entity.length as usize;
        let command = &text[offset..length];

        match self.execute_command(&update, command, &text[length..]) {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    fn handle_location(
        &mut self,
        _update: &Update,
        message: &Message,
        location: &Location,
    ) -> Result<(), HandleUpdateError> {
        let weather_data = get_weather(
            Identifier::Location(location.latitude, location.longitude),
            self.settings,
        )?;

        let mut message_params = SendMessageParams::new(
            ChatId::Integer(message.chat.id),
            format_weather_data(&weather_data),
        );
        message_params.set_reply_to_message_id(Some(message.message_id));

        self.api
            .send_message(&message_params)
            .map_err(HandleUpdateError::Api)
            .map(|_| ())
    }

    fn find_command_entity(message: &Message) -> Option<&MessageEntity> {
        message
            .entities
            .as_ref()?
            .iter()
            .find(|entity| entity.type_field.as_str() == BOT_COMMAND)
    }

    pub fn handle_update(&mut self, update: &Update) -> Result<(), HandleUpdateError> {
        let message = update
            .message
            .as_ref()
            .ok_or(HandleUpdateError::Skip(update.update_id))?;

        if let Some(err) = Self::find_command_entity(message).and_then(|entity| {
            // If there's a MessageEntity, there's some text which we can unwrap without panic
            self.handle_command(update, entity, message.text.as_ref().unwrap())
                .err()
        }) {
            match err {
                HandleUpdateError::Skip(_) => {}
                _ => return Err(err),
            };
        };

        if let Some(err) = message
            .location
            .as_ref()
            .and_then(|loc| self.handle_location(update, message, loc).err())
        {
            return Err(err);
        };

        Ok(())
    }
}
