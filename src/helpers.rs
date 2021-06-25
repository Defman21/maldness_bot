use std::env;

const ADMINS: Vec<String> = env!("ADMINS")
    .split(' ')
    .map(String::from)
    .collect();

pub fn is_admin(uid: u64) -> bool {
    ADMINS.contains(&uid.to_string())
}
