// src/models/gallery.rs

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::str::FromStr;
use strum::{Display, EnumIter, EnumString};
use time::OffsetDateTime;
use uuid::Uuid;

// NOWY ENUM: Musi odpowiadać temu w bazie danych
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, EnumString, EnumIter, Display)]
#[sqlx(type_name = "gallery_category", rename_all = "PascalCase")]
#[strum(serialize_all = "PascalCase")]
pub enum GalleryCategory {
    Piersi,
    Tyłek,
    Cipka,
    #[strum(serialize = "Całe Ciało")]
    CałeCiało,
    OtwieramCipkęDlaCiebie,
    #[strum(serialize = "Otwieram Cipkę dla Ciebie")]
    Analne,
    #[strum(serialize = "Zabawy wibratorem")]
    ZabawyWibratorem,
    Orgazm,
}

// --- DODAJ TEN BLOK KODU ---
// Ręcznie implementujemy konwersję ze String, delegując do istniejącej implementacji z &str.
impl TryFrom<String> for GalleryCategory {
    type Error = strum::ParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Używamy `from_str` z cechy `EnumString`, którą już zaimplementowaliśmy
        GalleryCategory::from_str(&value)
    }
}
// --- KONIEC NOWEGO BLOKU ---

#[derive(sqlx::FromRow, Clone, Serialize, Deserialize)]
pub struct Gallery {
    pub id: Uuid,
    pub erika_id: Uuid,
    #[sqlx(try_from = "String")]
    pub name: GalleryCategory,
    pub description: Option<String>,
    pub price_pln: Option<BigDecimal>,

    // WAŻNA ZMIANA: Używamy typu `time::OffsetDateTime`
    // Atrybut `serde(with ...)` mówi, jak serializować ten typ (to ważne dla API/sesji)
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl Gallery {
    pub async fn create(
        erika_id: Uuid,
        name: GalleryCategory,
        db: &PgPool,
    ) -> Result<Self, sqlx::Error> {
        let new_id = Uuid::new_v4();

        sqlx::query!(
            "INSERT INTO galleries (id, erika_id, name) VALUES ($1, $2, $3)",
            new_id,
            erika_id,
            // Rzutujemy nasz enum na typ, który sqlx rozumie
            name as GalleryCategory
        )
        .execute(db)
        .await?;

        // Używamy `query_as!` z jawnym typowaniem kolumny
        let new_gallery = sqlx::query_as!(
            Gallery,
            r#"SELECT id, erika_id, name as "name: _", description, price_pln, created_at FROM galleries WHERE id = $1"#,
            new_id
        )
        .fetch_one(db)
        .await?;

        Ok(new_gallery)
    }

    pub async fn find_by_erika_id(erika_id: Uuid, db: &PgPool) -> Result<Vec<Self>, sqlx::Error> {
        let galleries = sqlx::query_as!(
            Gallery,
            r#"SELECT id, erika_id, name as "name: _", description, price_pln, created_at FROM galleries WHERE erika_id = $1 ORDER BY created_at DESC"#,
            erika_id
        )
        .fetch_all(db)
        .await?;
        Ok(galleries)
    }

    // NOWA METODA: Aktualizuje szczegóły galerii
    pub async fn update_details(
        id: Uuid,
        name: GalleryCategory, // Zmieniamy typ na enum
        description: &str,
        price_pln: Option<BigDecimal>,
        db: &PgPool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE galleries SET name = $1, description = $2, price_pln = $3 WHERE id = $4",
            name as GalleryCategory, // Rzutujemy enum
            description,
            price_pln,
            id
        )
        .execute(db)
        .await?;
        Ok(())
    }

    pub async fn find_by_id(id: Uuid, db: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Gallery,
            r#"SELECT id, erika_id, name as "name: _", description, price_pln, created_at FROM galleries WHERE id = $1"#,
            id
        )
        .fetch_optional(db)
        .await
    }

    // NOWA METODA: Znajduje galerię tylko jeśli ID i właściciel się zgadzają
    pub async fn find_by_id_and_erika_id(
        id: Uuid,
        erika_id: Uuid,
        db: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Gallery,
            r#"SELECT id, erika_id, name as "name: _", description, price_pln, created_at FROM galleries WHERE id = $1 AND erika_id = $2"#,
            id,
            erika_id
        )
        .fetch_optional(db)
        .await
    }
}
