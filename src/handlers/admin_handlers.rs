// src/handlers/admin_handlers.rs
use crate::handlers::layout;
use crate::models::gallery::Gallery;
use crate::{app_state::AppState, errors::AppError, models::erika::Erika};
use axum::Form;
use axum::extract::Path;
use axum::response::Redirect;
use axum::{extract::State, response::Html};
use tracing::info;
use uuid::Uuid;

// Handler do wyświetlania głównego dashboardu admina
pub async fn admin_dashboard(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    // Na razie pobierzmy wszystkie Eriki
    let erikas = Erika::find_all(&state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    let content = maud::html! {
        h1 class="text-3xl font-bold text-white mb-6" { "Panel Administratora" }

        h2 class="text-xl font-semibold text-white mb-4" { "Lista Modelek" }
        div class="bg-gray-800 rounded-lg shadow-lg" {
            ul {
                @for erika in erikas {
                    li class="p-4 border-b border-gray-700 flex justify-between items-center" {
                        div {
                            span class="text-white" { (erika.username) " (" (erika.email) ")" }
                            // Wyświetlanie statusu
                            @if erika.is_approved {
                                span class="ml-4 text-xs font-semibold bg-green-500 text-white px-2 py-1 rounded-full" { "Zaakceptowana" }
                            } @else {
                                span class="ml-4 text-xs font-semibold bg-yellow-500 text-black px-2 py-1 rounded-full" { "Oczekuje" }
                            }
                        }
                        div class="flex gap-2" {
                            // Formularz z przyciskiem "Akceptuj", widoczny tylko dla niezaakceptowanych
                            @if !erika.is_approved {
                                form action=(format!("/admin/erika/{}/approve", erika.id)) method="post" {
                                    button type="submit" class="bg-green-600 hover:bg-green-700 text-white font-bold py-1 px-3 rounded-md text-sm" {
                                        "Akceptuj"
                                    }
                                }
                            }
                            a href=(format!("/admin/erika/{}", erika.id)) class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-1 px-3 rounded-md text-sm" {
                                "Edytuj"
                            }
                        }
                    }
                }
            }
        }
    };
    Ok(Html(layout::page("Admin", content).into_string()))
}

// NOWY HANDLER: Wyświetla stronę edycji profilu konkretnej Eriki
pub async fn show_edit_erika_form(
    Path(erika_id): Path<Uuid>, // Pobieramy ID z URL
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let erika = Erika::find_by_id(erika_id, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or(AppError::NotFound)?;

    let galleries = Gallery::find_by_erika_id(erika_id, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    let content = maud::html! {
        a href="/admin" class="inline-block mb-6 text-blue-400 hover:text-blue-300 transition-colors" {
            "← Wróć do listy"
        }
        h1 class="text-3xl font-bold text-white mb-6" { "Edytuj profil: " (erika.username) }

        // --- POPRAWKA TUTAJ: Wypełniamy formularz ---
        div class="bg-gray-800 p-6 rounded-lg shadow-lg" {
            form action=(format!("/admin/erika/{}", erika.id)) method="post" {
                div class="mb-4" {
                    label for="username" class="block text-gray-300 text-sm font-bold mb-2" { "Nazwa użytkownika:" }
                    input type="text" name="username" value=(erika.username) required
                          class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500";
                }
                div class="mb-4" {
                    label for="email" class="block text-gray-300 text-sm font-bold mb-2" { "Email:" }
                    input type="email" name="email" value=(erika.email) required
                          class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500";
                }
                div class="mb-6" {
                    label for="bio" class="block text-gray-300 text-sm font-bold mb-2" { "Krótkie bio:" }
                    textarea name="bio" rows="3"
                              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500" {
                        (erika.bio.as_deref().unwrap_or(""))
                    }
                }
                button type="submit" class="w-full bg-green-600 hover:bg-green-700 text-white font-bold py-2 px-4 rounded-md transition duration-300" { "Zapisz zmiany" }
            }
        }
        // --- KONIEC POPRAWKI ---

        h2 class="text-2xl font-semibold text-white mt-8 mb-4" { "Galerie tej modelki" }
        @if galleries.is_empty() {
            p class="text-gray-400" { "Brak galerii." }
        } @else {
            div class="space-y-4" {
                @for gallery in galleries {
                    div class="bg-gray-700 p-4 rounded-lg" {
                        p class="text-white" { (gallery.name.to_string()) }
                    }
                }
            }
        }
    };
    Ok(Html(layout::page("Edytuj Erikę", content).into_string()))
}

// NOWY HANDLER: Przetwarza formularz edycji
pub async fn update_erika_by_admin(
    Path(erika_id): Path<Uuid>,
    State(state): State<AppState>,
    Form(payload): Form<crate::handlers::erika_handlers::UpdateProfilePayload>, // Używamy ponownie tej struktury
) -> Result<Redirect, AppError> {
    // Wywołujemy istniejącą logikę aktualizacji, ale bez uploadu avatara
    Erika::update_profile_details(
        erika_id,
        &payload.username,
        &payload.email,
        &payload.bio,
        &state.db,
    )
    .await
    .map_err(|_| AppError::InternalServerError)?;

    info!("Admin zaktualizował profil dla: {}", erika_id);
    Ok(Redirect::to("/admin"))
}

// NOWY HANDLER: Akceptuje profil Eriki
pub async fn approve_erika(
    Path(erika_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Redirect, AppError> {
    Erika::approve(erika_id, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    info!("Admin zaakceptował profil: {}", erika_id);
    Ok(Redirect::to("/admin"))
}
