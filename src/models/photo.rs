// src/models/photo.rs
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow, Clone, Serialize, Deserialize)]
pub struct Photo {
    pub id: Uuid,
    pub gallery_id: Uuid,
    pub file_url: String,
    pub description: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl Photo {
    /// Zapisuje informacje o nowym zdjęciu w bazie danych.
    pub async fn create(
        gallery_id: Uuid,
        file_url: &str,
        db: &PgPool,
    ) -> Result<Self, sqlx::Error> {
        let photo = sqlx::query_as!(
            Photo,
            "INSERT INTO photos (gallery_id, file_url) VALUES ($1, $2) RETURNING *",
            gallery_id,
            file_url
        )
        .fetch_one(db)
        .await?;
        Ok(photo)
    }

    /// Pobiera wszystkie zdjęcia dla danej galerii.
    pub async fn find_by_gallery_id(
        gallery_id: Uuid,
        db: &PgPool,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let photos = sqlx::query_as!(
            Photo,
            "SELECT * FROM photos WHERE gallery_id = $1 ORDER BY created_at ASC",
            gallery_id
        )
        .fetch_all(db)
        .await?;
        Ok(photos)
    }

    pub async fn delete(id: Uuid, db: &PgPool) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM photos WHERE id = $1", id)
            .execute(db)
            .await?;
        Ok(())
    }
}
