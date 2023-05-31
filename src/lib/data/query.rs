//! Database queries.

use super::model;
use crate::data::{DataError, DatabasePool};
use crate::web::api::ApiKey;
use crate::ShortCode;
use sqlx::Row;

/// [`Result`] alias for database query functions.
type Result<T> = std::result::Result<T, DataError>;

/// Increases the hit count for the [`crate::domain::Job`] as identified by the [`ShortCode`].
pub async fn increase_hit_count(
    shortcode: &ShortCode,
    responses: u32,
    pool: &DatabasePool,
) -> Result<()> {
    let shortcode = shortcode.as_str();
    Ok(sqlx::query!(
        "UPDATE jobs SET responses = responses + ? WHERE shortcode = ?",
        responses,
        shortcode
    )
    .execute(pool)
    .await
    .map(|_| ())?)
}

/// Gets a [`Job`](`crate::domain::Job`).
pub async fn get_job<M: Into<model::GetJob>>(model: M, pool: &DatabasePool) -> Result<model::Job> {
    let model = model.into();
    let shortcode = model.shortcode.as_str();
    Ok(sqlx::query_as!(
        model::Job,
        "SELECT * FROM jobs WHERE shortcode = ?",
        shortcode
    )
    .fetch_one(pool)
    .await?)
}

/// Adds a [`Job`](`crate::domain::Job`).
pub async fn new_job<M: Into<model::NewJob>>(model: M, pool: &DatabasePool) -> Result<model::Job> {
    let model = model.into();
    let _ = sqlx::query!(
        r#"INSERT INTO jobs (
            job_id,
            shortcode,
            escrow_id,
            manifest_url,
            posted,
            expires,
            password,
            responses)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        model.job_id,
        model.shortcode,
        model.escrow_id,
        model.manifest_url,
        model.posted,
        model.expires,
        model.password,
        0
    )
    .execute(pool)
    .await?;
    get_job(model.shortcode, pool).await
}

/// Fetches latest GraphJobs.
pub async fn get_last_fetched_escrow_id_time(pool: &DatabasePool) -> Result<Option<i64>> {
    let row = sqlx::query!("SELECT posted FROM jobs ORDER BY posted DESC LIMIT 1")
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.posted))
}

/// Updates a [`Job`](`crate::domain::Job`).
pub async fn update_job<M: Into<model::UpdateJob>>(
    model: M,
    pool: &DatabasePool,
) -> Result<model::Job> {
    let model = model.into();
    let _ = sqlx::query!(
        r#"UPDATE jobs SET
            escrow_id = ?,
            expires = ?,
            password = ?,
            manifest_url = ?
           WHERE shortcode = ?"#,
        model.escrow_id,
        model.expires,
        model.password,
        model.manifest_url,
        model.shortcode
    )
    .execute(pool)
    .await?;
    get_job(model.shortcode, pool).await
}

/// Saves an [`ApiKey`].
pub async fn save_api_key(api_key: ApiKey, pool: &DatabasePool) -> Result<ApiKey> {
    let bytes = api_key.clone().into_inner();
    let _ = sqlx::query!("INSERT INTO api_keys (api_key) VALUES (?)", bytes)
        .execute(pool)
        .await
        .map(|_| ())?;
    Ok(api_key)
}

/// The return value from the [`revoke_api_key`] function.
pub enum RevocationStatus {
    /// The [`ApiKey`] was successfully revoked.
    Revoked,
    /// The [`ApiKey`] was not found, so no revocation occuured.
    NotFound,
}

/// Revokes an [`ApiKey`].
pub async fn revoke_api_key(api_key: ApiKey, pool: &DatabasePool) -> Result<RevocationStatus> {
    let bytes = api_key.clone().into_inner();
    Ok(
        sqlx::query!("DELETE FROM api_keys WHERE api_key == ?", bytes)
            .execute(pool)
            .await
            .map(|result| match result.rows_affected() {
                0 => RevocationStatus::NotFound,
                _ => RevocationStatus::Revoked,
            })?,
    )
}

/// Determines if the [`ApiKey`] is valid.
pub async fn api_key_is_valid(api_key: ApiKey, pool: &DatabasePool) -> Result<bool> {
    let bytes = api_key.clone().into_inner();
    Ok(
        sqlx::query("SELECT COUNT(api_key) FROM api_keys WHERE api_key = ?")
            .bind(bytes)
            .fetch_one(pool)
            .await
            .map(|row| {
                let count: u32 = row.get(0);
                count > 0
            })?,
    )
}

/// Deletes all expired [`Jobs`](`crate::domain::Job`).
pub async fn delete_expired(pool: &DatabasePool) -> Result<u64> {
    Ok(
        sqlx::query!(r#"DELETE FROM jobs WHERE strftime('%s', 'now') > expires"#)
            .execute(pool)
            .await?
            .rows_affected(),
    )
}

#[cfg(test)]
pub mod test {
    use crate::data::test::*;
    use crate::data::*;
    use crate::test::async_runtime;

    fn model_get_job(shortcode: &str) -> model::GetJob {
        model::GetJob {
            shortcode: shortcode.into(),
        }
    }

    fn model_new_job(shortcode: &str) -> model::NewJob {
        use chrono::Utc;
        model::NewJob {
            job_id: DbId::new().into(),
            escrow_id: format!("escrow_id for job '{}'", shortcode),
            manifest_url: None,
            shortcode: shortcode.into(),
            posted: Utc::now().timestamp(),
            expires: None,
            password: None,
        }
    }

    #[test]
    fn job_new_and_get() {
        let rt = async_runtime();
        let db = new_db(rt.handle());
        let pool = db.get_pool();

        let job =
            rt.block_on(async move { super::new_job(model_new_job("1"), &pool.clone()).await });
        assert!(job.is_ok());
        let job = job.unwrap();
        assert!(job.shortcode == "1");
        assert!(job.escrow_id == format!("escrow_id for job '1'"));
    }
}
