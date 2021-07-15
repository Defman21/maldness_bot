use std::collections::HashMap;
use std::sync::Mutex;

pub struct Cache {
    afk: Mutex<HashMap<i64, (bool, i32)>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            afk: Mutex::new(HashMap::new()),
        }
    }

    pub fn cache_afk_event_id(&self, user_id: i64, status: bool, event_id: i32) {
        let mut afk = self.afk.lock().unwrap();
        afk.insert(user_id, (status, event_id));
    }

    pub fn populate_afk_cache(&self, user_id_event_id_tuple: &[(i64, i32)]) {
        let mut afk = self.afk.lock().unwrap();
        for (user_id, event_id) in user_id_event_id_tuple.iter() {
            afk.insert(user_id.to_owned(), (true, event_id.to_owned()));
        }
    }

    pub fn get_afk_event_id(&self, user_id: i64) -> Option<i32> {
        let afk = self.afk.lock().unwrap();
        match afk.get(&user_id).unwrap_or(&(false, 0)) {
            (false, _) => None,
            (true, event_id) => Some(event_id.to_owned()),
        }
    }
}
