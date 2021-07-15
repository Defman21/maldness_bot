-- This file should undo anything in `up.sql`

alter table afk_events drop column event_type;

alter table afk_events rename to sleep_events;
