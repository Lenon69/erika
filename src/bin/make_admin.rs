// src/bin/make_admin.rs

use sqlx::postgres::PgPoolOptions;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("Nie znaleziono pliku .env");
    let database_url = env::var("DATABASE_URL").expect("Brak DATABASE_URL");

    // Pobieramy nazwę użytkownika z argumentów wiersza poleceń
    let args: Vec<String> = env::args().collect();
    let username = args
        .get(1)
        .expect("Podaj nazwę użytkownika, którego chcesz awansować na Admina!");

    let pool = PgPoolOptions::new().connect(&database_url).await?;

    // Zmieniamy rolę użytkownika w bazie danych
    let result = sqlx::query!(
        "UPDATE erikas SET role = 'Admin' WHERE username = $1",
        username
    )
    .execute(&pool)
    .await?;

    if result.rows_affected() > 0 {
        println!(
            "Sukces! Użytkownik '{}' jest teraz Administratorem.",
            username
        );
    } else {
        println!("Błąd: Nie znaleziono użytkownika o nazwie '{}'.", username);
    }

    Ok(())
}
