//! Actions that the service may perform.

use crate::data::{query, DatabasePool, Transaction};
use crate::service::ask;
use crate::web::api::ApiKey;
use crate::{Job, ServiceError, ShortCode};
use std::convert::TryInto;

/// Begins a new [`Transaction`].
pub async fn begin_transaction(pool: &DatabasePool) -> Result<Transaction<'_>, ServiceError> {
    Ok(pool.begin().await?)
}

/// Commits a [`Transaction`].
pub async fn end_transaction(transaction: Transaction<'_>) -> Result<(), ServiceError> {
    Ok(transaction.commit().await?)
}

/// Increases the number of responses for a [`Job`].
pub async fn increase_hit_count(
    shortcode: &ShortCode,
    responses: u32,
    pool: &DatabasePool,
) -> Result<(), ServiceError> {
    Ok(query::increase_hit_count(shortcode, responses, pool).await?)
}

/// Creates a new [`Job`].
pub async fn new_job(req: ask::NewJob, pool: &DatabasePool) -> Result<Job, ServiceError> {
    Ok(query::new_job(req, pool).await?.try_into()?)
}

/// Updates an existing [`Job`].
pub async fn update_job(req: ask::UpdateJob, pool: &DatabasePool) -> Result<Job, ServiceError> {
    Ok(query::update_job(req, pool).await?.try_into()?)
}

/// Gets a [`Job`].
pub async fn get_job(req: ask::GetJob, pool: &DatabasePool) -> Result<Job, ServiceError> {
    let user_password = req.password.clone();
    let job: Job = query::get_job(req, pool).await?.try_into()?;
    if job.password.has_password() {
        if job.password == user_password {
            Ok(job)
        } else {
            Err(ServiceError::PermissionError("Invalid password".to_owned()))
        }
    } else {
        Ok(job)
    }
}

/// Creates a new [`ApiKey`].
pub async fn generate_api_key(pool: &DatabasePool) -> Result<ApiKey, ServiceError> {
    let api_key = ApiKey::default();
    Ok(query::save_api_key(api_key, pool).await?)
}

/// Revokes an existing [`ApiKey`].
pub async fn revoke_api_key(
    api_key: ApiKey,
    pool: &DatabasePool,
) -> Result<query::RevocationStatus, ServiceError> {
    Ok(query::revoke_api_key(api_key, pool).await?)
}

/// Determines if an [`ApiKey`] is valid.
pub async fn api_key_is_valid(api_key: ApiKey, pool: &DatabasePool) -> Result<bool, ServiceError> {
    Ok(query::api_key_is_valid(api_key, pool).await?)
}

/// Deletes all expired [`Jobs`](`Job`).
pub async fn delete_expired(pool: &DatabasePool) -> Result<u64, ServiceError> {
    Ok(query::delete_expired(pool).await?)
}
