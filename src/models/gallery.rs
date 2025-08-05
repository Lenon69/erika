// src/models/gallery.rs

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(sqlx::FromRow, Clone, Serialize, Deserialize)]
pub struct Gallery {
    pub id: Uuid,
    pub erika_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price_pln: Option<BigDecimal>,

    // WAŻNA ZMIANA: Używamy typu `time::OffsetDateTime`
    // Atrybut `serde(with ...)` mówi, jak serializować ten typ (to ważne dla API/sesji)
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl Gallery {
    pub async fn create(erika_id: Uuid, name: &str, db: &PgPool) -> Result<Self, sqlx::Error> {
        let new_id = Uuid::new_v4();

        sqlx::query!(
            "INSERT INTO galleries (id, erika_id, name) VALUES ($1, $2, $3)",
            new_id,
            erika_id,
            name
        )
        .execute(db)
        .await?;

        let new_gallery = sqlx::query_as!(Gallery, "SELECT * FROM galleries WHERE id = $1", new_id)
            .fetch_one(db)
            .await?;

        Ok(new_gallery)
    }

    pub async fn find_by_erika_id(erika_id: Uuid, db: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let galleries = sqlx::query_as!(
            Gallery,
            "SELECT * FROM galleries WHERE erika_id = $1 ORDER BY created_at DESC",
            erika_id
        )
        .fetch_all(db)
        .await?;
        Ok(galleries)
    }
}
