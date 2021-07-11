-- Your SQL goes here

create table sleep_events
(
    id         serial
        constraint sleep_events_pk
            primary key,
    started_at timestamp not null,
    ended_at   timestamp,
    message    text,
    user_id    int       not null
        constraint sleep_events_users_id_fk
            references users
);
