use crate::data::DbId;
use derive_more::Constructor;
use serde::{Deserialize, Serialize};

/// The internal database id field for a [`Job`](crate::domain::job::Job).
#[derive(Clone, Debug, Constructor, Deserialize, Serialize)]
pub struct JobId(DbId);

impl JobId {
    /// Return the underlying [`DbId`](crate::data::DbId).
    pub fn into_inner(self) -> DbId {
        self.0
    }
}

impl From<DbId> for JobId {
    fn from(id: DbId) -> Self {
        Self(id)
    }
}

/// The Default implementation for for [`JobId`] is an empty ID.
impl Default for JobId {
    fn default() -> Self {
        Self(DbId::nil())
    }
}
