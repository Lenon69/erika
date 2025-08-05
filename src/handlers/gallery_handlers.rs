// src/handlers/gallery_handlers.rs
use super::layout;
use crate::{app_state::AppState, errors::AppError, models::gallery::Gallery};
use axum::{
    Form,
    extract::State,
    response::{Html, Redirect},
};
use serde::Deserialize;
use tower_sessions::Session;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateGalleryPayload {
    pub name: String,
}

// Handler do wyświetlania strony zarządzania galeriami
pub async fn show_galleries_page(
    session: Session,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let erika_id = session
        .get::<Uuid>("erika_id")
        .await
        .unwrap_or(None)
        .ok_or(AppError::Unauthorized)?;

    let galleries = Gallery::find_by_erika_id(erika_id, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    let content = maud::html! {
        div class="max-w-4xl mx-auto" {
            h1 class="text-3xl font-bold text-white mb-6" { "Zarządzaj swoimi galeriami" }

            // Formularz do tworzenia nowej galerii
            div class="bg-gray-800 p-6 rounded-lg shadow-lg mb-8" {
                h2 class="text-xl font-semibold text-white mb-4" { "Stwórz nową galerię" }
                form action="/panel/galleries" method="post" class="flex items-center gap-4" {
                    input type="text" name="name" placeholder="Nazwa galerii..." required
                          class="flex-grow px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500";
                    button type="submit" class="bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded-md transition duration-300" { "Stwórz" }
                }
            }

            // Lista istniejących galerii
            h2 class="text-xl font-semibold text-white mb-4" { "Twoje galerie" }
            @if galleries.is_empty() {
                p class="text-gray-400" { "Nie masz jeszcze żadnych galerii." }
            } @else {
                div class="space-y-4" {
                    @for gallery in galleries {
                        div class="bg-gray-800 p-4 rounded-lg flex justify-between items-center" {
                            p class="text-white" { (gallery.name) }
                            a href="#" class="text-blue-400 hover:underline" { "Zarządzaj zdjęciami" }
                        }
                    }
                }
            }
        }
    };
    Ok(Html(
        layout::page("Zarządzanie Galeriami", content).into_string(),
    ))
}

// Handler do przetwarzania formularza tworzenia galerii
pub async fn create_gallery(
    session: Session,
    State(state): State<AppState>,
    Form(payload): Form<CreateGalleryPayload>,
) -> Result<Redirect, AppError> {
    let erika_id = session
        .get::<Uuid>("erika_id")
        .await
        .unwrap_or(None)
        .ok_or(AppError::Unauthorized)?;

    Gallery::create(erika_id, &payload.name, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    Ok(Redirect::to("/panel/galleries"))
}
