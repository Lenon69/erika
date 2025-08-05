// src/handlers/erika_handlers.rs

use crate::models::gallery::Gallery;
use crate::{app_state::AppState, errors::AppError, models::erika::Erika};
use axum::extract::Multipart;
use axum::extract::Path as AxumPath;
use axum::{
    Form,
    extract::State,
    response::{Html, IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use sqlx::types::chrono;
use std::path::Path;
use tokio::fs;
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
            // --- NOWA SEKCJA: WYŚWIETLANIE AVATARA ---
            @if let Some(avatar_url) = &erika_data.profile_image_url {
                img src=(avatar_url) alt="Avatar" class="w-32 h-32 rounded-full mx-auto mb-6 object-cover border-4 border-blue-500";
            } @else {
                // Placeholder jeśli nie ma zdjęcia
                div class="w-32 h-32 rounded-full mx-auto mb-6 bg-gray-700 flex items-center justify-center border-4 border-gray-600" {
                    span class="text-gray-400" { "Brak zdjęcia" }
                }
            }

            h1 class="text-3xl font-bold text-white mb-2" { "Witaj w panelu, " (erika_data.username) "!" }
            p class="text-sm text-gray-400 mb-6" { "Twoje ID: " (erika_data.id) }


            // --- NOWE PRZYCISKI NAWIGACYJNE ---
            div class="flex items-center justify-center gap-4 mb-6" {
                a href="/panel/galleries" class="inline-block bg-purple-600 hover:bg-purple-700 text-white font-bold py-2 px-4 rounded-md transition duration-300" {
                    "Zarządzaj galeriami"
                }
                a href=(format!("/erika/{}", erika_data.username)) target="_blank" class="inline-block bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded-md transition duration-300" {
                    "Zobacz profil publiczny"
                }
                // Formularz do wylogowania
                form action="/logout" method="post" {
                    button type="submit" class="bg-red-600 hover:bg-red-700 text-white font-bold py-2 px-4 rounded-md transition duration-300" {
                        "Wyloguj"
                    }
                }
            }
            // --- KONIEC ---


            a href="/panel/galleries" class="inline-block bg-purple-600 hover:bg-purple-700 text-white font-bold py-2 px-4 rounded-md transition duration-300 mb-6" {
                "Zarządzaj galeriami"
            }
            hr class="border-gray-700 my-6";

            h2 class="text-2xl font-bold text-white mb-4" { "Edytuj swój profil" }
            // WAŻNE: Dodajemy enctype, aby formularz mógł wysyłać pliki
            form action="/panel" method="post" enctype="multipart/form-data" {
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

                div class="mb-6" {
                    label for="bio" class="block text-gray-300 text-sm font-bold mb-2" { "Krótkie bio:" }
                    textarea name="bio" rows="3"
                              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500" {
                        (erika_data.bio.as_deref().unwrap_or(""))
                    }
                }

                // --- NOWE POLE: WYBÓR PLIKU ---
                div class="mb-6" {
                    label for="avatar" class="block text-gray-300 text-sm font-bold mb-2" { "Zmień zdjęcie profilowe:" }
                    input type="file" id="avatar" name="avatar" accept="image/png, image/jpeg"
                          class="w-full text-sm text-gray-400 file:mr-4 file:py-2 file:px-4 file:rounded-full file:border-0 file:text-sm file:font-semibold file:bg-blue-600 file:text-white hover:file:bg-blue-700";
                }

                button type="submit"
                       class="w-full bg-green-600 hover:bg-green-700 text-white font-bold py-2 px-4 rounded-md transition duration-300" { "Zapisz zmiany" }
            }
        }
    };
    Ok(Html(layout::page("Panel Eriki", content).into_string()))
}

// NOWY handler do aktualizacji profilu (obsługuje pliki)
pub async fn update_erika_profile(
    session: Session,
    State(state): State<AppState>,
    mut multipart: Multipart, // Używamy ekstraktora Multipart
) -> Result<Redirect, AppError> {
    let erika_id = match session.get::<Uuid>("erika_id").await {
        Ok(Some(id)) => id,
        _ => return Err(AppError::InternalServerError),
    };

    let mut username = String::new();
    let mut email = String::new();
    let mut bio = String::new(); // <-- DODAJ ZMIENNĄ
    let mut avatar_url: Option<String> = None;

    // Przetwarzamy każdą część formularza multipart
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();
        let file_name = field.file_name().unwrap_or("").to_string();
        let data = field.bytes().await.unwrap();

        match name.as_str() {
            "username" => username = String::from_utf8(data.to_vec()).unwrap_or_default(),
            "email" => email = String::from_utf8(data.to_vec()).unwrap_or_default(),
            "bio" => bio = String::from_utf8(data.to_vec()).unwrap_or_default(),
            "avatar" if !data.is_empty() => {
                // Tworzymy unikalną nazwę pliku, aby uniknąć konfliktów
                let extension = Path::new(&file_name)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("jpg");
                let unique_filename = format!(
                    "{}_{}.{}",
                    erika_id,
                    chrono::Utc::now().timestamp(),
                    extension
                );
                let file_path_str = format!("uploads/{}", unique_filename);

                // Zapisujemy plik na serwerze
                fs::write(&file_path_str, &data)
                    .await
                    .map_err(|_| AppError::InternalServerError)?;
                info!("Zapisano nowy avatar: {}", file_path_str);

                // Zapisujemy publiczny URL, a nie ścieżkę systemową
                avatar_url = Some(format!("/{}", file_path_str));
            }
            _ => {}
        }
    }

    // Wywołujemy zaktualizowaną metodę z modelu
    Erika::update_profile(erika_id, &username, &email, &bio, avatar_url, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    info!("Zaktualizowano profil dla: {}", erika_id);
    Ok(Redirect::to("/panel"))
}

// Handler do wyświetlania strony głównej
pub async fn homepage(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    let erikas = Erika::find_active(&state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    let content = maud::html! {
        div class="max-w-7xl mx-auto" {
            h1 class="text-4xl font-bold text-white mb-8 text-center" { "Poznaj nasze modelki" }

            @if erikas.is_empty() {
                p class="text-gray-400 text-center" { "Obecnie żadna modelka nie jest dostępna." }
            } @else {
                div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6" {
                    @for erika in erikas {
                        // Link do przyszłej strony profilowej
                        a href=(format!("/erika/{}", erika.username)) class="bg-gray-800 rounded-lg overflow-hidden shadow-lg transform hover:-translate-y-1 transition-transform duration-300 block" {
                            div class="relative" {
                                img src=(erika.profile_image_url.as_deref().unwrap_or("/placeholder.jpg")) alt=(erika.username) class="w-full h-80 object-cover";
                                // Wskaźnik statusu online
                                @if erika.is_online {
                                    span class="absolute top-3 right-3 block w-4 h-4 bg-green-500 rounded-full border-2 border-gray-800" title="Online" {}
                                }
                            }
                            div class="p-4" {
                                h3 class="text-xl font-bold text-white" { (erika.username) }
                                p class="text-gray-400 mt-1 h-12 overflow-hidden" { (erika.bio.as_deref().unwrap_or("Brak opisu.")) }
                            }
                        }
                    }
                }
            }
        }
    };
    Ok(Html(layout::page("Strona Główna", content).into_string()))
}

// NOWY HANDLER: Wyświetla publiczną stronę profilową Eriki
pub async fn show_erika_profile(
    AxumPath(username): AxumPath<String>, // Pobieramy nazwę z URL
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    // 1. Znajdź Erikę w bazie po jej publicznej nazwie
    let erika = Erika::find_by_public_username(&username, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or(AppError::NotFound)?; // Zwróci błąd 404, jeśli nie ma takiej Eriki

    // 2. Znajdź wszystkie jej galerie
    let galleries = Gallery::find_by_erika_id(erika.id, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    // 3. Renderuj stronę
    let content = maud::html! {
        div class="max-w-4xl mx-auto" {
            // Sekcja profilu
            div class="bg-gray-800 p-8 rounded-lg shadow-lg flex flex-col md:flex-row items-center gap-8 mb-8" {
                img src=(erika.profile_image_url.as_deref().unwrap_or("/placeholder.jpg")) alt=(erika.username)
                    class="w-48 h-48 rounded-full object-cover border-4 border-purple-500 flex-shrink-0";
                div {
                    h1 class="text-4xl font-bold text-white" { (erika.username) }
                    @if erika.is_online {
                        span class="text-green-400 font-semibold" { "● Online" }
                    } @else {
                        span class="text-gray-400 font-semibold" { "● Offline" }
                    }
                    p class="text-gray-300 mt-4" { (erika.bio.as_deref().unwrap_or("Brak opisu.")) }
                }
            }

            // Sekcja galerii
            h2 class="text-3xl font-bold text-white mb-6" { "Płatne galerie" }
            @if galleries.is_empty() {
                p class="text-gray-400" { "Ta modelka nie udostępniła jeszcze żadnych galerii." }
            } @else {
                div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6" {
                    @for gallery in galleries {
                        div class="bg-gray-800 rounded-lg overflow-hidden shadow-lg" {
                            // W przyszłości tutaj będzie okładka galerii
                            div class="w-full h-56 bg-gray-700 flex items-center justify-center" {
                                span class="text-gray-500" { "Okładka galerii" }
                            }
                            div class="p-4 flex justify-between items-center" {
                                div {
                                    h3 class="text-lg font-bold text-white" { (gallery.name) }
                                    // WYŚWIETLANIE CENY
                                    @if let Some(price) = &gallery.price_pln {
                                        p class="text-green-400 font-bold" { (price.with_scale(2).to_string()) " PLN" }
                                    } @else {
                                        p class="text-gray-400" { "Darmowa" }
                                    }
                                }
                                // Zmieniamy link na przycisk płatności
                                a href=(format!("/pay/gallery/{}", gallery.id)) class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-md text-sm transition duration-300" {
                                    "Odblokuj"
                                }
                            }
                        }
                    }
                }
            }
        }
    };
    Ok(Html(layout::page(&erika.username, content).into_string()))
}

pub async fn initiate_gallery_payment(
    AxumPath(gallery_id): AxumPath<Uuid>,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let gallery = Gallery::find_by_id(gallery_id, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or(AppError::NotFound)?;

    let price_str = gallery
        .price_pln
        .as_ref()
        .map(|p| p.with_scale(2).to_string())
        .unwrap_or_else(|| "0.00".to_string());

    let message = format!(
        "Zamierzasz odblokować galerię '{}' za {} PLN.",
        gallery.name, price_str
    );

    let page = layout::info_page(
        "Potwierdzenie płatności",
        &message,
        Some(("#", "Zapłać z PayU")),
    );
    Ok(Html(page.into_string()))
}

// NOWY HANDLER: Obsługuje wylogowanie
pub async fn logout(session: Session) -> Result<Redirect, AppError> {
    // Czyścimy sesję, usuwając wszystkie zapisane w niej dane
    session.clear();
    info!("Użytkownik pomyślnie wylogowany.");
    // Przekierowujemy na stronę główną
    Ok(Redirect::to("/"))
}
