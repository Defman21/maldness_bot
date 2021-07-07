-- Your SQL goes here

create table users
(
    id           serial                not null
        constraint users_pk
            primary key,
    telegram_uid bigint                not null,
    is_paying    boolean default false not null
);
