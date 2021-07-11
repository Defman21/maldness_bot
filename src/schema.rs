// @generated automatically by Diesel CLI.

diesel::table! {
    sleep_events (id) {
        id -> Int4,
        started_at -> Timestamp,
        ended_at -> Nullable<Timestamp>,
        message -> Nullable<Text>,
        user_id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        telegram_uid -> Int8,
        is_paying -> Bool,
        latitude -> Nullable<Float8>,
        longitude -> Nullable<Float8>,
    }
}

diesel::joinable!(sleep_events -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(sleep_events, users,);
