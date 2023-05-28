//! Data structures to make a service request.

use crate::domain::job::field;
use crate::ShortCode;

use serde::{Deserialize, Serialize};

/// Data required to run the [`new_job`](crate::service::action::new_job()) action to add a new [`crate::domain::Job`].
#[derive(Debug, Deserialize, Serialize)]
pub struct NewJob {
    pub escrow_id: field::EscrowId,
    pub manifest_id: field::ManifestId,
    pub posted: field::Posted,
    pub expires: field::Expires,
    pub password: field::Password,
}

/// Data required to run the [`update_job`](crate::service::action::update_job()) action to update [`crate::domain::Job`] data.
#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateJob {
    pub escrow_id: field::EscrowId,
    pub manifest_id: field::ManifestId,
    pub expires: field::Expires,
    pub password: field::Password,
    pub shortcode: field::ShortCode,
}

/// Data required to run the [`get_job`](crate::service::action::get_job()) action to get a [`crate::domain::Job`].
#[derive(Debug, Deserialize, Serialize)]
pub struct GetJob {
    pub shortcode: ShortCode,
    pub password: field::Password,
}

impl GetJob {
    /// Convert a [`&str`] into a [`GetJob`] action request.
    pub fn from_raw(shortcode: &str) -> Self {
        Self {
            shortcode: ShortCode::from(shortcode),
            password: field::Password::default(),
        }
    }
}

impl From<ShortCode> for GetJob {
    fn from(shortcode: ShortCode) -> Self {
        Self {
            shortcode,
            password: field::Password::default(),
        }
    }
}

impl From<&str> for GetJob {
    fn from(raw: &str) -> Self {
        Self::from_raw(raw)
    }
}
