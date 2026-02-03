use std::time::SystemTime;

use crate::created_at;

pub trait Identifiable {
    fn id(&self) -> &str;

    fn created_at(&self) -> SystemTime {
        created_at(self.id())
    }
}

#[cfg(feature = "either")]
impl<L: Identifiable, R: Identifiable> Identifiable for either::Either<L, R> {
    fn id(&self) -> &str {
        match self {
            either::Either::Left(l) => l.id(),
            either::Either::Right(r) => r.id(),
        }
    }
}
