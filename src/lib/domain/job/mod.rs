//! Structures, errors, and implementation for the [`Job`](crate::Job) data type.
pub mod field;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The possible errors that can occur when building a [`Job`]
#[derive(Debug, Error)]
pub enum JobError {
    /// Password does not meet complexity requirements.
    #[error("invalid password: {0}")]
    InvalidPassword(String),

    /// Job manifest_id has unwanted words/data.
    #[error("invalid manifest_id: {0}")]
    InvalidManifestId(String),

    /// EscrowId was not provided.
    #[error("empty escrow_id")]
    EmptyEscrowId,

    /// Date is invalid: invalid day of the month, too far in the past, etc.
    #[error("invalid date: {0}")]
    InvalidDate(String),

    /// Date failed to parse.
    #[error("date parse error: {0}")]
    DateParse(#[from] chrono::ParseError),

    /// [crate::data::DbId] failed to parse.
    #[error("id parse error: {0}")]
    Id(#[from] uuid::Error),

    /// Number of responses is negative or not a number.
    #[error("responses parse error: {0}")]
    Responses(#[from] std::num::TryFromIntError),
}

/// Job stores all the data about Jobs posted to the service.
///
/// Each field in the Job uses a newtype that encapsulates the requirements
/// for that particular field. If one of the fields cannot be created, then
/// a Job cannot be created. This enforcement of field creation ensures
/// that a Job will always be valid whenever it is utilized at any point
/// in the program.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Job {
    #[serde(skip)]
    /// The internal [`DbId`](crate::data::DbId) for the Job.
    pub job_id: field::JobId,
    /// The code used to access this job from the service.
    pub shortcode: field::ShortCode,
    /// The escrow_id of the Job.
    pub escrow_id: field::EscrowId,
    /// The manifest_id of the Job.
    pub manifest_id: field::ManifestId,
    /// The date that this Job was posted to the service.
    pub posted: field::Posted,
    /// The date that this Job will expire.
    pub expires: field::Expires,
    /// The password needed to view this Job.
    pub password: field::Password,
    /// The number of responses received by this Job.
    pub responses: field::Responses,
}
