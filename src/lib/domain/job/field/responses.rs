use derive_more::Constructor;
use serde::{Deserialize, Serialize};

/// The responses field for a [`Job`](crate::domain::job::Job).
#[derive(Clone, Constructor, Debug, Deserialize, Serialize)]
pub struct Responses(u64);

impl Responses {
    /// Return the underlying [`u64`].
    pub fn into_inner(self) -> u64 {
        self.0
    }
}
