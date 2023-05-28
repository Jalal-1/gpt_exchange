-- Add migration script here
create TABLE IF NOT EXISTS jobs
(
    job_id   TEXT PRIMARY KEY NOT NULL,
    shortcode TEXT UNIQUE NOT NULL,
    escrow_id   TEXT NOT NULL,
    manifest_id     TEXT,
    posted    DATETIME NOT NULL,
    expires   DATETIME,
    password  TEXT,
    responses      BIGINT NOT NULL
);

CREATE TABLE IF NOT EXISTS api_keys
(
    api_key BLOB PRIMARY KEY
);
