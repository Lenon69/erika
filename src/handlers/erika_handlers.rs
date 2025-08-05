// src/handlers/erika_handlers.rs

use crate::{app_state::AppState, errors::AppError, models::erika::Erika};
use axum::{
    Form,
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use tower_sessions::Session;
use tracing::{info, warn};
use uuid::Uuid;

// Importujemy nasz moduł layoutu
use super::layout;

#[derive(Deserialize)]
pub struct RegisterErikaPayload {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct UpdateProfilePayload {
    pub username: String,
    pub email: String,
}

// Handler formularza rejestracji
pub async fn show_register_form() -> Html<String> {
    let content = maud::html! {
        div class="max-w-md mx-auto bg-gray-800 p-8 rounded-lg shadow-lg" {
            h1 class="text-3xl font-bold text-white mb-6 text-center" { "Zarejestruj się" }
            form action="/register" method="post" {
                div class="mb-4" {
                    label for="username" class="block text-gray-300 text-sm font-bold mb-2" { "Nazwa użytkownika:" }
                    input type="text" id="username" name="username" required
                          class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500";
                }
                div class="mb-4" {
                    label for="email" class="block text-gray-300 text-sm font-bold mb-2" { "Email:" }
                    input type="email" id="email" name="email" required
                            class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500";
                }
                div class="mb-6" {
                    label for="password" class="block text-gray-300 text-sm font-bold mb-2" { "Hasło:" }
                    input type="password" id="password" name="password" required
                          class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500";
                }
                button type="submit"
                       class="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-md transition duration-300" { "Zarejestruj" }
            }
        }
    };
    Html(layout::page("Rejestracja", content).into_string())
}

// Handler przetwarzania rejestracji
pub async fn register_erika(
    State(state): State<AppState>,
    Form(payload): Form<RegisterErikaPayload>,
) -> Result<Redirect, AppError> {
    // Lepsze UX: Przekierowanie zamiast komunikatu
    match Erika::create(&payload, &state.db).await {
        Ok(_) => {
            info!("Zarejestrowano pomyślnie użytkownika: {}", payload.username);
            // Po udanej rejestracji, przekieruj na stronę logowania
            Ok(Redirect::to("/login"))
        }
        Err(_) => Err(AppError::InternalServerError),
    }
}

// Handler formularza logowania (już go zrobiliśmy, ale jest tu dla spójności)
pub async fn show_login_form() -> Html<String> {
    let content = maud::html! {
        div class="max-w-md mx-auto bg-gray-800 p-8 rounded-lg shadow-lg" {
            h1 class="text-3xl font-bold text-white mb-6 text-center" { "Zaloguj się" }
            form action="/login" method="post" {
                div class="mb-4" {
                    label for="username" class="block text-gray-300 text-sm font-bold mb-2" { "Nazwa użytkownika:" }
                    input type="text" id="username" name="username" required
                          class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500";
                }
                div class="mb-6" {
                    label for="password" class="block text-gray-300 text-sm font-bold mb-2" { "Hasło:" }
                    input type="password" id="password" name="password" required
                          class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500";
                }
                button type="submit"
                       class="w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-md transition duration-300" { "Zaloguj" }
            }
        }
    };
    Html(layout::page("Logowanie", content).into_string())
}

// Handler przetwarzania logowania
pub async fn login_erika(
    State(state): State<AppState>,
    session: Session,
    Form(payload): Form<LoginPayload>,
) -> Result<Response, AppError> {
    // Zmieniamy typ zwracany
    info!("Próba logowania dla użytkownika: {}", payload.username);

    let erika_result = Erika::find_by_username(&payload.username, &state.db).await;

    match erika_result {
        Ok(Some(erika)) if erika.verify_password(&payload.password) => {
            info!("Weryfikacja hasła powiodła się.");
            session
                .insert("erika_id", erika.id)
                .await
                .map_err(|_| AppError::InternalServerError)?;
            // Po udanym logowaniu, przekieruj do panelu
            Ok(Redirect::to("/panel").into_response())
        }
        _ => {
            warn!("Logowanie nie powiodło się dla: {}", payload.username);
            // Używamy naszej nowej strony z informacją o błędzie
            let error_page = layout::info_page(
                "Błąd Logowania",
                "Nieprawidłowe dane logowania.",
                Some(("/login", "Spróbuj ponownie")),
            );
            Ok(Html(error_page.into_string()).into_response())
        }
    }
}

// Handler panelu Eriki
pub async fn erika_panel(
    session: Session,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let erika_id = match session.get::<Uuid>("erika_id").await {
        Ok(Some(id)) => id,
        _ => {
            let page = layout::info_page(
                "Brak dostępu",
                "Musisz się zalogować, aby zobaczyć tę stronę.",
                Some(("/login", "Przejdź do logowania")),
            );
            return Ok(Html(page.into_string()));
        }
    };

    let erika_data = match Erika::find_by_id(erika_id, &state.db).await {
        Ok(Some(data)) => data,
        _ => return Err(AppError::InternalServerError),
    };

    let content = maud::html! {
        div class="max-w-2xl mx-auto bg-gray-800 p-8 rounded-lg shadow-lg" {
            h1 class="text-3xl font-bold text-white mb-2" { "Witaj w panelu, " (erika_data.username) "!" }
            p class="text-sm text-gray-400 mb-6" { "Twoje ID: " (erika_data.id) }
            hr class="border-gray-700 my-6";

            h2 class="text-2xl font-bold text-white mb-4" { "Edytuj swój profil" }
            form action="/panel" method="post" {
                div class="mb-4" {
                    label for="username" class="block text-gray-300 text-sm font-bold mb-2" { "Nazwa użytkownika:" }
                    input type="text" name="username" value=(erika_data.username) required
                          class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500";
                }
                div class="mb-6" {
                    label for="email" class="block text-gray-300 text-sm font-bold mb-2" { "Email:" }
                    input type="email" name="email" value=(erika_data.email) required
                          class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500";
                }
                button type="submit"
                       class="w-full bg-green-600 hover:bg-green-700 text-white font-bold py-2 px-4 rounded-md transition duration-300" { "Zapisz zmiany" }
            }
        }
    };
    Ok(Html(layout::page("Panel Eriki", content).into_string()))
}

// Handler aktualizacji profilu (bez zmian w logice, zwraca Redirect)
pub async fn update_erika_profile(
    session: Session,
    State(state): State<AppState>,
    Form(payload): Form<UpdateProfilePayload>,
) -> Result<Redirect, AppError> {
    let erika_id = match session.get::<Uuid>("erika_id").await {
        Ok(Some(id)) => id,
        _ => return Err(AppError::InternalServerError),
    };

    Erika::update_profile(erika_id, &payload.username, &payload.email, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    info!("Zaktualizowano profil dla: {}", erika_id);
    Ok(Redirect::to("/panel"))
}
