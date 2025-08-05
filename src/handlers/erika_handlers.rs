use axum::{Form, extract::State, response::Html};
use maud::{DOCTYPE, html};
use serde::Deserialize;
use tower_sessions::Session;
use tracing::info;
use uuid::Uuid;

use crate::{app_state::AppState, errors::AppError, models::erika::Erika};

// Struktura, która dokładnie odpowiada polom 'name' w formularzu HTML
#[derive(Deserialize)]
pub struct RegisterErikaPayload {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub async fn show_register_form() -> Html<String> {
    Html(
        html! {
            (DOCTYPE)
            html {
                head {
                    title { "Rejestracja Nowej Modelki" }
                }
                body {
                    h1 { "Zarejestruj się jako nowa modelka" }
                    form action="/register" method="post" {
                        label for="username" { "Nazwa użytkownika:" }
                        input type="text" id="username" name="username" required;
                        br;
                        label for="email" { "Email:" }
                        input type="email" id="email" name="email" required;
                        br;
                        label for="password" { "Hasło:" }
                        input type="password" id="password" name="password" required;
                        br;
                        button type="submit" { "Zarejestruj" }
                    }
                }
            }
        }
        .into_string(),
    )
}

// Nowy handler do obsługi POST
pub async fn register_erika(
    State(state): State<AppState>,
    Form(payload): Form<RegisterErikaPayload>,
) -> Result<Html<&'static str>, AppError> {
    // Wywołujemy naszą logikę z modelu.
    // To tutaj zniknie pierwsze ostrzeżenie, bo używamy `state.db`!
    match Erika::create(&payload, &state.db).await {
        Ok(_) => {
            info!("Zarejestrowano pomyślnie użytkownika: {}", payload.username);
            Ok(Html("<h1>Rejestracja pomyślna!</h1>"))
        }
        Err(_) => {
            // A tutaj zniknie drugie ostrzeżenie, bo w końcu konstruujemy nasz błąd!
            Err(AppError::InternalServerError)
        }
    }
}

#[derive(Deserialize)]
pub struct LoginPayload {
    username: String,
    password: String,
}

// Handler do wyświetlania formularza logowania
pub async fn show_login_form() -> Html<String> {
    Html(
        maud::html! {
            (maud::DOCTYPE)
            html {
                head { title { "Logowanie" } }
                body {
                    h1 { "Zaloguj się" }
                    form action="/login" method="post" {
                        label for="username" { "Nazwa użytkownika:" }
                        input type="text" id="username" name="username" required;
                        br;
                        label for="password" { "Hasło:" }
                        input type="password" id="password" name="password" required;
                        br;
                        button type="submit" { "Zaloguj" }
                    }
                }
            }
        }
        .into_string(),
    )
}

// Handler do przetwarzania logowania
pub async fn login_erika(
    State(state): State<AppState>,
    session: Session,
    Form(payload): Form<LoginPayload>,
) -> Result<Html<&'static str>, AppError> {
    let erika = Erika::find_by_username(&payload.username, &state.db).await;

    match erika {
        Ok(Some(erika)) if erika.verify_password(&payload.password) => {
            // Hasło poprawne - tworzymy sesję
            session
                .insert("erika_id", erika.id)
                .await
                .map_err(|_| AppError::InternalServerError)?;
            info!("Zalogowano pomyślnie użytkownika: {}", payload.username);
            Ok(Html(
                "<h1>Zalogowano pomyślnie!</h1><p><a href=\"/panel\">Przejdź do panelu</a></p>",
            ))
        }
        _ => {
            // Użytkownik nie znaleziony lub hasło niepoprawne
            Ok(Html(
                "<h1>Nieprawidłowe dane logowania.</h1><p><a href=\"/login\">Spróbuj ponownie</a></p>",
            ))
        }
    }
}

// Prosty, chroniony handler panelu
// ZMODYFIKOWANY handler do wyświetlania panelu (GET)
pub async fn erika_panel(
    session: Session,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    // Sprawdzamy ID w sesji
    let erika_id = match session.get::<Uuid>("erika_id").await {
        Ok(Some(id)) => id,
        _ => {
            return Ok(Html(
                "<h1>Brak dostępu. Musisz się zalogować.</h1>".to_string(),
            ));
        }
    };

    // Pobieramy pełne dane Eriki z bazy
    let erika_data = match Erika::find_by_id(erika_id, &state.db).await {
        Ok(Some(data)) => data,
        _ => return Err(AppError::InternalServerError), // Błąd lub brak użytkownika
    };

    // Renderujemy szablon Maud z danymi i formularzem
    let response = maud::html! {
        (maud::DOCTYPE)
        html {
            head { title { "Panel Eriki" } }
            body {
                h1 { "Witaj w panelu, " (erika_data.username) "!" }
                p { "Twoje ID: " (erika_data.id) }
                hr;
                h2 { "Edytuj swój profil" }
                form action="/panel" method="post" {
                    label for="username" { "Nazwa użytkownika:" }
                    // Wypełniamy pole aktualną wartością
                    input type="text" name="username" value=(erika_data.username) required;
                    br;
                    // Emaila na razie nie mamy w struct Erika, więc zostawiamy puste
                    // W przyszłości to uzupełnimy
                    label for="email" { "Email:" }
                    input type="email" name="email" value="" required; // Uzupełnimy to
                    br;
                    button type="submit" { "Zapisz zmiany" }
                }

                // W przyszłości dodamy tutaj link do wylogowania
            }
        }
    };

    Ok(Html(response.into_string()))
}

// NOWY handler do aktualizacji profilu (POST)
pub async fn update_erika_profile(
    session: Session,
    State(state): State<AppState>,
    Form(payload): Form<UpdateProfilePayload>,
) -> Result<axum::response::Redirect, AppError> {
    // Zwracamy Redirect

    let erika_id = match session.get::<Uuid>("erika_id").await {
        Ok(Some(id)) => id,
        _ => return Err(AppError::InternalServerError), // Nieautoryzowana próba
    };

    Erika::update_profile(erika_id, &payload.username, &payload.email, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    info!("Zaktualizowano profil dla: {}", erika_id);

    // Przekierowujemy z powrotem do panelu, aby zobaczyć zmiany
    Ok(axum::response::Redirect::to("/panel"))
}

// Struktura do odbioru danych z formularza edycji
#[derive(Deserialize)]
pub struct UpdateProfilePayload {
    username: String,
    email: String,
}
