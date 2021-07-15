-- Your SQL goes here

alter table sleep_events
    rename to afk_events;

alter table afk_events
    add column event_type int not null default 1;
