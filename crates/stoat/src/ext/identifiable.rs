use std::time::SystemTime;

pub trait Identifiable {
    fn created_at(&self) -> SystemTime;
}
