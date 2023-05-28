//! Database models for executing queries & returning data.

use crate::data::DbId;
use crate::{JobError, ShortCode, Time};
use chrono::NaiveDateTime;
use std::convert::TryFrom;
use std::u64;

/// Job that is stored in, and retrieved from, the database.
#[derive(Debug, sqlx::FromRow)]
pub struct Job {
    pub(in crate::data) job_id: String,
    pub(in crate::data) shortcode: String,
    pub(in crate::data) escrow_id: String,
    pub(in crate::data) manifest_id: Option<String>,
    pub(in crate::data) posted: i64,
    pub(in crate::data) expires: Option<NaiveDateTime>,
    pub(in crate::data) password: Option<String>,
    pub(in crate::data) responses: i64,
}

/// Convert from a database model Job into a domain Job.
impl TryFrom<Job> for crate::domain::Job {
    type Error = JobError;
    fn try_from(job: Job) -> Result<Self, Self::Error> {
        use crate::domain::job::field;
        use std::str::FromStr;
        Ok(Self {
            job_id: field::JobId::new(DbId::from_str(job.job_id.as_str())?),
            shortcode: field::ShortCode::from(job.shortcode),
            escrow_id: field::EscrowId::new(job.escrow_id.as_str())?,
            manifest_id: field::ManifestId::new(job.manifest_id),
            posted: field::Posted::new(u64::try_from(job.posted)?),
            expires: field::Expires::new(job.expires.map(Time::from_naive_utc)),
            password: field::Password::new(job.password.unwrap_or_default())?,
            responses: field::Responses::new(u64::try_from(job.responses)?),
        })
    }
}

/// Data required to run the [`get_job`](crate::data::query::get_job()) query to get a [`Job`] from the database.
pub struct GetJob {
    pub(in crate::data) shortcode: String,
}

impl From<crate::service::ask::GetJob> for GetJob {
    fn from(req: crate::service::ask::GetJob) -> Self {
        Self {
            shortcode: req.shortcode.into_inner(),
        }
    }
}

impl From<ShortCode> for GetJob {
    fn from(shortcode: ShortCode) -> Self {
        GetJob {
            shortcode: shortcode.into_inner(),
        }
    }
}

impl From<String> for GetJob {
    fn from(shortcode: String) -> Self {
        GetJob { shortcode }
    }
}

/// Data required to run the [`new_job`](crate::data::query::new_job()) query to add a [`Job`] to the database.
pub struct NewJob {
    pub(in crate::data) job_id: String,
    pub(in crate::data) shortcode: String,
    pub(in crate::data) escrow_id: String,
    pub(in crate::data) manifest_id: Option<String>,
    pub(in crate::data) posted: i64,
    pub(in crate::data) expires: Option<i64>,
    pub(in crate::data) password: Option<String>,
}

impl From<crate::service::ask::NewJob> for NewJob {
    fn from(req: crate::service::ask::NewJob) -> Self {
        Self {
            job_id: DbId::new().into(),
            escrow_id: req.escrow_id.into_inner(),
            manifest_id: req.manifest_id.into_inner(),
            expires: req.expires.into_inner().map(|time| time.timestamp()),
            password: req.password.into_inner(),
            shortcode: ShortCode::default().into(),
            posted: req.posted.into_inner() as i64,
        }
    }
}

/// Data required to run the [`update_job`](crate::data::query::update_job()) query to update a [`Job`] in the database.
pub struct UpdateJob {
    pub(in crate::data) shortcode: String,
    pub(in crate::data) escrow_id: String,
    pub(in crate::data) manifest_id: Option<String>,
    pub(in crate::data) expires: Option<i64>,
    pub(in crate::data) password: Option<String>,
}

impl From<crate::service::ask::UpdateJob> for UpdateJob {
    fn from(req: crate::service::ask::UpdateJob) -> Self {
        Self {
            escrow_id: req.escrow_id.into_inner(),
            manifest_id: req.manifest_id.into_inner(),
            expires: req.expires.into_inner().map(|time| time.timestamp()),
            password: req.password.into_inner(),
            shortcode: ShortCode::default().into(),
        }
    }
}
