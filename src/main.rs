mod app_state;
mod errors;
mod handlers;
mod models;
mod router;

use app_state::AppState;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::info;

use time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    dotenvy::dotenv().expect("Nie znaleziono pliku .env");

    let database_url = std::env::var("DATABASE_URL").expect("Brak DATABASE_URL w .env");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    info!("Połączenie z bazą danych nawiązane pomyślnie!");

    // Konfiguracja magazynu sesji w PostgreSQL
    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?; // Automatycznie tworzy tabelę, jeśli nie istnieje

    // Konfiguracja warstwy sesji - sesja wygasa po 1 dniu
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false) // Ustaw na 'true' gdy przejdziesz na HTTPS
        .with_expiry(Expiry::OnInactivity(Duration::days(1)));

    // Tworzymy router i dodajemy do niego warstwę sesji
    let app_state = AppState { db: pool };

    let app = router::create_router(app_state).layer(session_layer);
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let listener = TcpListener::bind(addr).await?;
    info!("Serwer nasłuchuje na http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
