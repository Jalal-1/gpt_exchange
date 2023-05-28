-- Add migration script here
CREATE TABLE IF NOT EXISTS jobs
(
    job_id   TEXT PRIMARY KEY NOT NULL,
    shortcode TEXT UNIQUE NOT NULL,
    escrow_id   TEXT NOT NULL,
    manifest_id TEXT,
    posted    BIGINT NOT NULL,
    expires   DATETIME,
    password  TEXT,
    responses      BIGINT NOT NULL
);
