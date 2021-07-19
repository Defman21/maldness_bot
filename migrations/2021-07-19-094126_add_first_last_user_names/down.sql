-- This file should undo anything in `up.sql`

alter table users
    drop column first_name;

alter table users
    drop column last_name;

alter table users
    drop column username;
