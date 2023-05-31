use crate::domain::job::JobError;
use rocket::form::{self, FromFormField, ValueField};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// The manifest_url field for a [`Job`](crate::domain::job::Job).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ManifestUrl(Option<String>);

impl ManifestUrl {
    /// Create a new `ManifestUrl` field.
    pub fn new<T: Into<Option<String>>>(manifest_url: T) -> Self {
        let manifest_url: Option<String> = manifest_url.into();
        match manifest_url {
            Some(manifest_url) => {
                if !manifest_url.trim().is_empty() {
                    Self(Some(manifest_url))
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

/// The Default implementation is no manifest_url.
impl Default for ManifestUrl {
    fn default() -> Self {
        Self::new(None)
    }
}

impl FromStr for ManifestUrl {
    type Err = JobError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.to_string()))
    }
}

#[rocket::async_trait]
impl<'r> FromFormField<'r> for ManifestUrl {
    fn from_value(field: ValueField<'r>) -> form::Result<'r, Self> {
        Ok(Self::new(field.value.to_owned()))
    }
}

#[cfg(test)]
mod test {
    use super::ManifestUrl;

    #[test]
    fn blank_manifest_url_converts_to_none() {
        assert!(ManifestUrl::new("".to_owned()).into_inner().is_none());
    }

    #[test]
    fn valid_manifest_url_allowed() {
        assert!(ManifestUrl::new("manifest_url".to_owned())
            .into_inner()
            .is_some());
    }
}
