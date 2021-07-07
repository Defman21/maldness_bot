use diesel::table;

table! {
    users (id) {
        id -> Int4,
        telegram_uid -> Int8,
        is_paying -> Bool,
    }
}
