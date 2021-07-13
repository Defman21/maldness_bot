use std::error::Error;
use std::process::exit;
use std::time::Duration;

use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::PgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use frankenstein::{
    Api, ChatId, Location, Message, MessageEntity, SendMessageParams, SetMyCommandsParams,
    TelegramApi, Update,
};

use crate::cache::Cache;
use crate::commands::CommandsExecutor;
use crate::errors::HandleUpdateError;
use crate::helpers;
use crate::services::sleep::functions::end_sleep;
use crate::services::sleep::{
    errors::ServiceError, format_sleep_data, functions::get_sleeping_users,
};
use crate::services::weather::{format_weather_data, get_weather, Identifier};
use crate::settings::Settings;

const BOT_COMMAND: &str = "bot_command";

pub struct UpdateHandler<'a> {
    api: &'a Api,
    settings: &'a Settings,
    pub commands_executor: CommandsExecutor<'a>,
    bot_prefix: String,
    postgres: PgConnection,
    cache: &'a Cache,
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

fn run_migrations(
    conn: &mut impl MigrationHarness<Pg>,
) -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    conn.run_pending_migrations(MIGRATIONS)?;

    Ok(())
}

impl<'a> UpdateHandler<'a> {
    pub fn new(api: &'a Api, settings: &'a Settings, cache: &'a Cache) -> UpdateHandler<'a> {
        let mut handler = UpdateHandler {
            api,
            settings,
            commands_executor: CommandsExecutor::new(settings, api, cache),
            bot_prefix: String::new(),
            postgres: PgConnection::establish(settings.postgres_dsn.as_str())
                .expect("Failed to connect to postgres"),
            cache,
        };

        if let Err(err) = run_migrations(&mut handler.postgres) {
            println!("Failed to run the migrations: {}", err);
            exit(1);
        };

        match api.get_me() {
            Ok(response) => {
                let prefix = response
                    .result
                    .username
                    .expect("Failed to get the username");
                handler.set_bot_prefix(prefix);
            }
            Err(error) => {
                panic!("Failed to get myself: {:?}", error);
            }
        };

        match get_sleeping_users(&mut handler.postgres) {
            Ok(ref sleepers) => handler.cache.populate_sleep_cache(&sleepers),
            Err(err) => panic!("Failed to populate sleep cache from DB: {:?}", err),
        }

        handler
    }

    pub fn set_bot_prefix(&mut self, prefix: String) {
        self.bot_prefix = prefix;
    }

    pub fn send_my_commands(&self) {
        let set_my_commands_params =
            SetMyCommandsParams::new(self.commands_executor.bot_commands());
        self.api
            .set_my_commands(&set_my_commands_params)
            .expect("Failed to set my commands");
    }

    fn handle_command(
        &mut self,
        update: &Update,
        message: &Message,
        command_entity: &MessageEntity,
    ) -> Result<(), HandleUpdateError> {
        let text = message.text.as_ref().unwrap();
        let offset = command_entity.offset as usize;

        if offset != 0 {
            return Ok(());
        }

        let length = command_entity.length as usize;
        let command = &text[offset..offset + length];

        match self.commands_executor.execute(
            self.bot_prefix.as_str(),
            &mut self.postgres,
            &update,
            command,
            &message,
            &text[length..],
        ) {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    fn handle_location(
        &self,
        _update: &Update,
        message: &Message,
        Location {
            latitude,
            longitude,
            ..
        }: &Location,
    ) -> Result<(), HandleUpdateError> {
        let weather_data = get_weather(
            Identifier::Location {
                latitude: *latitude,
                longitude: *longitude,
            },
            self.settings,
        )?;

        let mut message_params = SendMessageParams::new(
            ChatId::Integer(message.chat.id),
            format_weather_data(&weather_data, &self.settings),
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
        let message = update.message.as_ref().ok_or(HandleUpdateError::Skip)?;

        if let Some(err) = self.settings.check_for_allowed_update(message) {
            return Err(err);
        }

        let user_id = helpers::get_user_id(message)?;
        if self.cache.get_sleep_status(user_id) {
            match end_sleep(user_id, &mut self.postgres) {
                Ok(event) => {
                    let sleep_duration = Duration::from_secs(
                        (event.ended_at.unwrap() - event.started_at)
                            .to_std()
                            .unwrap()
                            .as_secs(),
                    );
                    let _ = helpers::send_text_message(
                        self.api,
                        message.chat.id,
                        format_sleep_data(
                            self.settings,
                            message.from.as_ref().unwrap(),
                            event.message,
                            sleep_duration,
                        ),
                        Some(message.message_id),
                    );
                }
                Err(ServiceError::NotFound) => {}
                Err(err) => println!("Failed to end sleep status for user {}: {}", user_id, err),
            };
            self.cache.cache_sleep_status(user_id, false);
        }

        if let Some(err) = Self::find_command_entity(message).and_then(|entity| {
            // If there's a MessageEntity, there's some text which we can unwrap without panic
            self.handle_command(update, message, entity).err()
        }) {
            match err {
                HandleUpdateError::Skip => {}
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
