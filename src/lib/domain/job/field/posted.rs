use derive_more::Constructor;
use rocket::form::{self, FromFormField, ValueField};
use serde::{Deserialize, Serialize};

/// The date posted field for a [`Job`](crate::domain::job::Job).
#[derive(Clone, Constructor, Debug, Deserialize, Serialize)]
pub struct Posted(u64);

impl Posted {
    /// Return the underlying blocktime.
    pub fn into_inner(self) -> u64 {
        self.0
    }
}

#[rocket::async_trait]
impl<'r> FromFormField<'r> for Posted {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        Ok(Self::new(field.value.parse::<u64>().unwrap()))
    }
}
