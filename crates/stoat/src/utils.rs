use std::time::SystemTime;

use ulid::Ulid;

pub fn created_at(id: &str) -> SystemTime {
    Ulid::from_string(id).expect("Malformed ID").datetime()
}
