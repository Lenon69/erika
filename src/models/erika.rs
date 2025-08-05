use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use serde::Serialize;
use sqlx::PgPool;
use tokio::task;
use tracing::debug;
use uuid::Uuid;

#[derive(sqlx::FromRow, Clone, Serialize)]
pub struct Erika {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
}

impl Erika {
    pub async fn create(
        payload: &super::super::handlers::erika_handlers::RegisterErikaPayload,
        db: &PgPool,
    ) -> Result<(), sqlx::Error> {
        let password_to_hash = payload.password.clone();

        // **OPTYMALIZACJA**: Nadal wykonujemy hashowanie w osobnym wątku.
        let password_hash = task::spawn_blocking(move || -> String {
            // 1. Generujemy unikalną, losową sól dla każdego hasła.
            let salt = SaltString::generate(&mut OsRng);

            // 2. Używamy domyślnej, bezpiecznej konfiguracji Argon2.
            let argon2 = Argon2::default();

            // 3. Hashujemy hasło z użyciem nowej soli.
            // Wynikiem jest kompletny hash zawierający wszystkie potrzebne informacje.
            let hash = argon2
                .hash_password(password_to_hash.as_bytes(), &salt)
                .expect("Hashowanie hasła nie powiodło się");

            // 4. Zwracamy hash jako string do zapisu w bazie.
            hash.to_string()
        })
        .await
        .expect("Zadanie hashowania w tle nie powiodło się");

        let new_id = Uuid::new_v4();

        // Zapis do bazy danych pozostaje bez zmian.
        sqlx::query!(
            "INSERT INTO erikas (id, username, email, password_hash) VALUES ($1, $2, $3, $4)",
            new_id, // <-- Przekazujemy wygenerowane ID
            payload.username,
            payload.email,
            password_hash
        )
        .execute(db)
        .await?;

        Ok(())
    }

    /// Wyszukuje użytkownika w bazie po jego nazwie.
    pub async fn find_by_username(
        username: &str,
        db: &PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Erika,
            "SELECT id, username, email, password_hash FROM erikas WHERE LOWER(username) = LOWER($1)",
            username
        )
        .fetch_optional(db)
        .await
    }

    /// Weryfikuje podane hasło z hashem zapisanym w bazie.
    pub fn verify_password(&self, password: &str) -> bool {
        debug!(
            "Rozpoczynam weryfikację hasła dla użytkownika: {}",
            self.username
        ); // <-- LOG A

        // Parsujemy hash zapisany w bazie
        let parsed_hash = match argon2::password_hash::PasswordHash::new(&self.password_hash) {
            Ok(hash) => {
                debug!("Hash z bazy poprawnie sparsowany."); // <-- LOG B (sukces)
                hash
            }
            Err(e) => {
                debug!("BŁĄD parsowania hasha z bazy: {}", e); // <-- LOG B (błąd)
                return false;
            }
        };

        // Weryfikujemy hasło
        let result = Argon2::default().verify_password(password.as_bytes(), &parsed_hash);

        match result {
            Ok(_) => {
                debug!("WYNIK: Hasło jest poprawne."); // <-- LOG C (sukces)
                true
            }
            Err(e) => {
                debug!("WYNIK: Hasło jest niepoprawne. Błąd weryfikacji: {}", e); // <-- LOG C (błąd)
                false
            }
        }
    }

    /// Wyszukuje Erikę po jej unikalnym ID (pobranym z sesji).
    pub async fn find_by_id(id: Uuid, db: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        sqlx::query_as!(
            Erika,
            "SELECT id, username, email, password_hash FROM erikas WHERE id = $1",
            id
        )
        .fetch_optional(db)
        .await
    }

    /// Aktualizuje profil Eriki w bazie danych.
    pub async fn update_profile(
        id: Uuid,
        username: &str,
        email: &str, // Dodajemy email do aktualizacji
        db: &PgPool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE erikas SET username = $1, email = $2 WHERE id = $3",
            username,
            email,
            id
        )
        .execute(db)
        .await?;

        Ok(())
    }
}
