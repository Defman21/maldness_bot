-- This file should undo anything in `up.sql`

alter table users
    drop column latitude;

alter table users
    drop column longitude;
