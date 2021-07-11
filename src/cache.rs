use std::collections::HashMap;
use std::sync::Mutex;

pub struct Cache {
    sleep: Mutex<HashMap<i64, bool>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            sleep: Mutex::new(HashMap::new()),
        }
    }

    pub fn cache_sleep_status(&self, user_id: i64, status: bool) {
        let mut sleep = self.sleep.lock().unwrap();
        sleep.insert(user_id, status);
    }

    pub fn populate_sleep_cache(&self, user_ids: &[i64]) {
        let mut sleep = self.sleep.lock().unwrap();
        for user_id in user_ids.iter() {
            sleep.insert(*user_id, true);
        }
    }

    pub fn get_sleep_status(&self, user_id: i64) -> bool {
        let sleep = self.sleep.lock().unwrap();
        *sleep.get(&user_id).unwrap_or(&false)
    }
}
