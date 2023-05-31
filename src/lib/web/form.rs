//! Form data.

use crate::domain::job::field;
use rocket::form::FromForm;
use serde::Serialize;

/// The form to create a new [`Job`](crate::Job).
#[derive(Debug, Serialize, FromForm)]
pub struct NewJob {
    pub escrow_id: field::EscrowId,
    pub manifest_url: field::ManifestUrl,
    pub posted: field::Posted,
    pub expires: field::Expires,
    pub password: field::Password,
}

/// The form to submit a [`Password`](crate::domain::job::field::Password) for a protected [`Job`](crate::Job).
#[derive(Debug, Serialize, FromForm)]
pub struct GetPasswordProtectedJob {
    pub password: field::Password,
}
