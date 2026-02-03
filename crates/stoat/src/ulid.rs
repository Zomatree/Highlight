use std::{ops::Deref, time::SystemTime};

use crate::{Error, Identifiable, Result};

pub struct Ulid(String);

impl Ulid {
    pub fn from_string(id: String) -> Result<Self> {
        if ulid::Ulid::from_string(&id).is_err() {
            return Err(Error::MalformedID);
        };

        Ok(Self(id))
    }

    pub fn from_string_unchecked(id: String) -> Self {
        Self(id)
    }

    pub fn timestamp(&self) -> SystemTime {
        ulid::Ulid::from_string(&self.0).unwrap().datetime()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn to_string(self) -> String {
        self.0
    }
}

impl Deref for Ulid {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Identifiable for Ulid {
    fn id(&self) -> &str {
        &self.0
    }
}
