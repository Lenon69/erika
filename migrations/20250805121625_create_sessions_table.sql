-- migrations/YYYYMMDDHHMMSS_create_sessions_table.sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY NOT NULL,
    data BYTEA NOT NULL,
    expiry TIMESTAMPTZ
);
