-- migrations/YYYY..._add_bio_and_status_to_erikas.sql
ALTER TABLE erikas
ADD COLUMN bio TEXT,
ADD COLUMN is_online BOOLEAN NOT NULL DEFAULT FALSE;
