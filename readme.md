# Maldness Bot

This is a Telegram bot I'm working on in my free time to learn Rust.

# Building

`docker build -t .` should be enough.

# External dependencies

* PostgreSQL

# Configuration

> Until 2021.1 (aka the first stable release), I change configuration options very frequently, so expect the provided config to be broken!

```toml
# telegram bot token
token = "1234567:ABCDEFG"
# List of user ids with admin privileges 
admins = [123, 456]
# PostgreSQL dsn string
postgres_dsn = "postgresql://admin:123@postgres/maldness_bot?sslmode=disable" 
# Wake-up message format, available variables:
# * username - telegram username
# * message - optional text message they've left when went to bed (N/A if none)
# * duration - human-readable duration (1h 2m 3s)
wake_up_format = "{{ username }} woke up and said: {{ message }}. He've slept for {{ duration }}"
# Back from work message format, available variables are the same as in wake_up_format
back_from_work_format = "{{ username }} finished working: {{ message }}. They've worked for {{ duration }}"

[allowed_chats]
# allow unspecified chats to use the bot, defaults to true
allow_unspecified = false
# a map of <chat_id> = <is_allowed>
supergroup = {-1234567 = true}
group = {}
# 0 means "*" or every chat of the following type
private = {0 = true}

[open_weather]
# OpenWeather API key
api_key = "abcde"
# OpenWeather units: could be standard (Kelvin), metric (Celsius) and imperial (Fahrenheit)
units = "metric"
# OpenWeather language: https://openweathermap.org/current#multi
language = "en"
# /weather message format, available variables:
# * name - Geolocation name
# * temp - Current temperature in the location
# * feels_like - Current "feels like" in the location
# * description - human-readable description with some emoji indicating current weather
message_format = "{{ name }}: {{ temp }} (feels like {{ feels_like }}), {{ description }}"

[commands.donate]
# Text for the /donate command
text = "https://patreon.com/defman"

[commands.weather]
# Text for when the bot could not find weather data for a location
not_found_text = "No weather data for this location found"
# Text for when the user does not have a location set but tries to call /weather without arguments
no_location_text = "You don't have a location set. Send me a geolocation message and call /set_my_location on it."
# Text for when someone tries to look up other's user location forecast, and they don't have a location set
no_location_for_user_text = "This user does not have a location set."

[commands.gn]
# Text for the /gn command
good_night_text = "Good night!"

[commands.shuffle]
# Text for when the bot could not shuffle anything in the message or in the reply to the message
nothing_to_shuffle_text = "Nothing to shuffle!"

[commands.work]
# Text for the /work command
work_text = "Have a good one, king."

[commands.rafk]
# Text for /rafk command when there's no afk event for the user
no_afk_event_text = "You haven't been afk, tho..."
```
