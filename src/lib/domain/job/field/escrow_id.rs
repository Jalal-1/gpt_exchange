use crate::domain::job::JobError;
use rocket::form::{self, FromFormField, ValueField};
use serde::{Deserialize, Serialize};

/// The escrow_id field for a [`Job`](crate::domain::job::Job).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EscrowId(String);

impl EscrowId {
    /// Create a new `EscrowId` field.
    ///
    /// If the escrow_id provided is empty, then a [`JobError`] will be returned.
    pub fn new(escrow_id: &str) -> Result<Self, JobError> {
        if !escrow_id.trim().is_empty() {
            Ok(Self(escrow_id.to_owned()))
        } else {
            Err(JobError::EmptyEscrowId)
        }
    }
    /// Return the underlying [`String`].
    pub fn into_inner(self) -> String {
        self.0
    }
    /// Return a reference to the underlying [`&str`].
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[rocket::async_trait]
impl<'r> FromFormField<'r> for EscrowId {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        Ok(Self::new(field.value).map_err(|e| form::Error::validation(format!("{}", e)))?)
    }
}

#[cfg(test)]
mod test {
    use super::EscrowId;

    #[test]
    fn disallow_empty_escrow_id() {
        assert!(EscrowId::new("").is_err());
    }
}
