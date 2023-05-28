use crate::domain::job::JobError;
use rocket::form::{self, FromFormField, ValueField};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// The manifest_id field for a [`Job`](crate::domain::job::Job).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ManifestId(Option<String>);

impl ManifestId {
    /// Create a new `ManifestId` field.
    pub fn new<T: Into<Option<String>>>(manifest_id: T) -> Self {
        let manifest_id: Option<String> = manifest_id.into();
        match manifest_id {
            Some(manifest_id) => {
                if !manifest_id.trim().is_empty() {
                    Self(Some(manifest_id))
                } else {
                    Self(None)
                }
            }
            None => Self(None),
        }
    }

    /// Return the underlying [`Option<String>`](`String`).
    pub fn into_inner(self) -> Option<String> {
        self.0
    }
}

/// The Default implementation is no manifest_id.
impl Default for ManifestId {
    fn default() -> Self {
        Self::new(None)
    }
}

impl FromStr for ManifestId {
    type Err = JobError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.to_string()))
    }
}

#[rocket::async_trait]
impl<'r> FromFormField<'r> for ManifestId {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        Ok(Self::new(field.value.to_owned()))
    }
}

#[cfg(test)]
mod test {
    use super::ManifestId;

    #[test]
    fn blank_manifest_id_converts_to_none() {
        assert!(ManifestId::new("".to_owned()).into_inner().is_none());
    }

    #[test]
    fn valid_manifest_id_allowed() {
        assert!(ManifestId::new("manifest_id".to_owned())
            .into_inner()
            .is_some());
    }
}
