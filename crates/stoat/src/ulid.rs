use std::{ops::Deref, time::SystemTime};

use crate::{Error, Identifiable, Result};

/// Wrapper around a ULID string, the ID format used for Stoat models.
pub struct Ulid(String);

impl Ulid {
    /// Verifies the input is a valid ULID and creates an instance of [`Ulid`].
    pub fn from_string(id: String) -> Result<Self> {
        if ulid::Ulid::from_string(&id).is_err() {
            return Err(Error::MalformedID);
        };

        Ok(Self(id))
    }

    /// Creates an instance of [`Ulid`] **_without verifying it is valid_**.
    ///
    /// Do not use this function unless you already know the input is valid.
    pub fn from_string_unchecked(id: String) -> Self {
        Self(id)
    }

    /// Returns the stored timestamp of when the ID was created.
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

impl TryFrom<String> for Ulid {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        Ulid::from_string(value)
    }
}
