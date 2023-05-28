//! Models for executing graph queries & returning data.
use crate::data::query::get_last_fetched_escrow_id_time;
use crate::data::DatabasePool;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GraphJob {
    pub id: String,
    pub timestamp: String,
}

/// Graph response object for [`launchedEscrows`] query.
#[derive(Debug, Deserialize)]
struct Data {
    launchedEscrows: Vec<GraphJob>,
}

#[derive(Debug, Deserialize)]
struct Error {
    message: String,
}

// Graph query response
#[derive(Debug, Deserialize)]
struct QueryResponse {
    data: Option<Data>,
    // errors: Option<Vec<Error>>,
}

pub async fn get_escrows_from_graph() -> Result<Vec<GraphJob>, Box<dyn std::error::Error>> {
    let query = r#"
        {
            launchedEscrows(first: 500, orderBy: timestamp, orderDirection: desc) {
                id
                timestamp
           }
        }
    "#;

    let client = reqwest::Client::new();

    let res = client
        .post("https://api.thegraph.com/subgraphs/name/humanprotocol/mumbai-v1")
        .json(&json!({
            "query": query,
        }))
        .send()
        .await?
        .json::<QueryResponse>()
        .await?;

    match res.data {
        None => Err("Response from server did not contain any data".into()),

        Some(data) => Ok(data.launchedEscrows),
    }
}

pub async fn fetch_new_jobs_from_graph(
    pool: &DatabasePool,
) -> Result<Vec<GraphJob>, Box<dyn std::error::Error>> {
    let last_escrow_id_time = get_last_fetched_escrow_id_time(pool).await?;

    let query = format!(
        r#"
        {{
            launchedEscrows(
                first: 50,
                orderBy: timestamp,
                orderDirection: desc,
                {}
            ) {{
                id,
                timestamp
            }}
        }}
    "#,
        match last_escrow_id_time {
            Some(posted) => format!(r#"where: {{ timestamp_gt: "{}" }}"#, posted),
            None => "".to_string(),
        }
    );

    let client = reqwest::Client::new();

    let res = client
        .post("https://api.thegraph.com/subgraphs/name/humanprotocol/mumbai-v1")
        .json(&json!({
            "query": query,
        }))
        .send()
        .await?
        .json::<QueryResponse>()
        .await?;

    match res.data {
        None => Err("Response from server did not contain any data".into()),

        Some(data) => Ok(data.launchedEscrows),
    }
}
