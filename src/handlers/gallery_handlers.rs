// src/handlers/gallery_handlers.rs
use super::layout;
use crate::models::photo::Photo;
use crate::{app_state::AppState, errors::AppError, models::gallery::Gallery};
use axum::extract::{Multipart, Path as AxumPath};
use axum::{
    Form,
    extract::State,
    response::{Html, Redirect},
};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use std::path::Path;
use tokio::fs;
use tower_sessions::Session;
use tracing::info;
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
            a href="/panel" class="inline-block mb-6 text-blue-400 hover:text-blue-300 transition-colors" {
                "← Wróć do panelu"
            }
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
                            a href=(format!("/panel/galleries/{}", gallery.id)) class="text-blue-400 hover:underline" { "Zarządzaj zdjęciami" }
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

// Handler do wyświetlania strony zarządzania JEDNĄ galerią
pub async fn show_single_gallery_page(
    AxumPath(gallery_id): AxumPath<Uuid>, // Pobieramy ID galerii z URL
    session: Session,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    let _erika_id = session
        .get::<Uuid>("erika_id")
        .await
        .unwrap_or(None)
        .ok_or(AppError::Unauthorized)?;

    // TODO: W przyszłości warto sprawdzić, czy ta galeria na pewno należy do zalogowanej Eriki.

    // Pobieramy dane galerii, żeby wypełnić formularz
    let gallery = Gallery::find_by_id(gallery_id, &state.db)
        .await // <-- POTRZEBUJEMY TEJ FUNKCJI
        .map_err(|_| AppError::InternalServerError)?
        .ok_or(AppError::NotFound)?;

    let photos = Photo::find_by_gallery_id(gallery_id, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    let content = maud::html! {
        div class="max-w-4xl mx-auto" {
            a href="/panel/galleries" class="inline-block mb-6 text-blue-400 hover:text-blue-300 transition-colors" {
                "← Wróć do listy galerii"
            }
            h1 class="text-3xl font-bold text-white mb-6" { "Zarządzaj zdjęciami w galerii" }

            // Formularz do wgrywania zdjęć
            div class="bg-gray-800 p-6 rounded-lg shadow-lg mb-8" {
                h2 class="text-xl font-semibold text-white mb-4" { "Dodaj nowe zdjęcie" }
                // Zwróć uwagę na dynamiczny URL w `action`
                form action={"/panel/galleries/" (gallery_id) "/upload"} method="post" enctype="multipart/form-data" {
                    input type="file" name="photo" required accept="image/png, image/jpeg"
                          class="w-full text-sm text-gray-400 file:mr-4 file:py-2 file:px-4 file:rounded-full file:border-0 file:text-sm file:font-semibold file:bg-blue-600 file:text-white hover:file:bg-blue-700";
                    button type="submit" class="mt-4 w-full bg-green-600 hover:bg-green-700 text-white font-bold py-2 px-4 rounded-md transition duration-300" { "Wgraj zdjęcie" }
                }
            }

            // NOWY FORMULARZ: Edycja danych galerii
            div class="bg-gray-800 p-6 rounded-lg shadow-lg mb-8" {
                h2 class="text-xl font-semibold text-white mb-4" { "Edytuj szczegóły galerii" }
                form action={"/panel/galleries/" (gallery.id)} method="post" {
                    div class="mb-4" {
                        label for="name" class="block text-gray-300 text-sm font-bold mb-2" { "Nazwa galerii:" }
                        input type="text" name="name" value=(gallery.name) required
                              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white ...";
                    }
                    div class="mb-4" {
                        label for="description" class="block text-gray-300 text-sm font-bold mb-2" { "Opis:" }
                        textarea name="description" rows="3"
                                  class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white ..." {
                            (gallery.description.as_deref().unwrap_or(""))
                        }
                    }
                    div class="mb-6" {
                        label for="price_pln" class="block text-gray-300 text-sm font-bold mb-2" { "Cena (PLN):" }
                        input type="number" name="price_pln" step="0.01" placeholder="np. 19.99" value=[gallery.price_pln.as_ref().map(|p| p.with_scale(2).to_string())]
                              class="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white ...";
                    }
                    button type="submit" class="w-full bg-purple-600 hover:bg-purple-700 text-white font-bold py-2 px-4 rounded-md ..."{ "Zapisz szczegóły" }
                }
            }


            // Siatka z wgranymi zdjęciami
            h2 class="text-xl font-semibold text-white mb-4" { "Zdjęcia w tej galerii" }
            @if photos.is_empty() {
                p class="text-gray-400" { "Brak zdjęć w tej galerii." }
            } @else {
                div class="grid grid-cols-2 md:grid-cols-4 gap-4" {
                    @for photo in photos {
                        div class="bg-gray-800 rounded-lg overflow-hidden shadow-lg" {
                            img src=(photo.file_url) alt="Zdjęcie z galerii" class="w-full h-48 object-cover";

                            // Formularz, który pojawi się po najechaniu na zdjęcie
                            form action=(format!("/panel/galleries/{}/photo/{}/delete", gallery_id, photo.id)) method="post"
                                 class="absolute inset-0 bg-black/60 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity" {
                                button type="submit" class="bg-red-600 hover:bg-red-700 text-white font-bold py-2 px-4 rounded-md" {
                                    "Usuń"
                                }
                            }
                        }
                    }
                }
            }
        }
    };
    Ok(Html(
        layout::page("Zarządzanie Galerią", content).into_string(),
    ))
}

// Handler do przetwarzania uploadu zdjęcia
pub async fn upload_photo(
    AxumPath(gallery_id): AxumPath<Uuid>,
    session: Session,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Redirect, AppError> {
    let erika_id = session
        .get::<Uuid>("erika_id")
        .await
        .unwrap_or(None)
        .ok_or(AppError::Unauthorized)?;

    while let Some(field) = multipart.next_field().await.unwrap() {
        if field.name() == Some("photo") {
            let file_name = field.file_name().unwrap_or("photo.jpg").to_string();
            let data = field.bytes().await.unwrap();

            if !data.is_empty() {
                let extension = Path::new(&file_name)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("jpg");
                let unique_filename = format!(
                    "{}_{}.{}",
                    Uuid::new_v4(),
                    chrono::Utc::now().timestamp(),
                    extension
                );
                let file_path_str = format!("uploads/{}", unique_filename);

                fs::write(&file_path_str, &data)
                    .await
                    .map_err(|_| AppError::InternalServerError)?;

                let public_url = format!("/{}", file_path_str);
                Photo::create(gallery_id, &public_url, &state.db)
                    .await
                    .map_err(|_| AppError::InternalServerError)?;

                info!(
                    "Dodano nowe zdjęcie do galerii {} przez Erikę {}",
                    gallery_id, erika_id
                );
            }
        }
    }

    Ok(Redirect::to(&format!("/panel/galleries/{}", gallery_id)))
}

#[derive(Deserialize)]
pub struct UpdateGalleryPayload {
    name: String,
    description: String,
    price_pln: Option<BigDecimal>,
}

// NOWY HANDLER: Aktualizuje dane galerii
pub async fn update_gallery(
    AxumPath(gallery_id): AxumPath<Uuid>,
    session: Session,
    State(state): State<AppState>,
    Form(payload): Form<UpdateGalleryPayload>,
) -> Result<Redirect, AppError> {
    let _erika_id = session
        .get::<Uuid>("erika_id")
        .await
        .unwrap_or(None)
        .ok_or(AppError::Unauthorized)?;

    // TODO: Weryfikacja, czy galeria należy do zalogowanej Eriki

    Gallery::update_details(
        gallery_id,
        &payload.name,
        &payload.description,
        payload.price_pln,
        &state.db,
    )
    .await
    .map_err(|_| AppError::InternalServerError)?;

    Ok(Redirect::to(&format!("/panel/galleries/{}", gallery_id)))
}

pub async fn delete_photo(
    AxumPath((gallery_id, photo_id)): AxumPath<(Uuid, Uuid)>,
    session: Session,
    State(state): State<AppState>,
) -> Result<Redirect, AppError> {
    let _erika_id = session
        .get::<Uuid>("erika_id")
        .await
        .unwrap_or(None)
        .ok_or(AppError::Unauthorized)?;

    // TODO: Weryfikacja, czy zdjęcie na pewno należy do zalogowanej Eriki

    // Usunięcie z bazy danych
    Photo::delete(photo_id, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    // TODO: Usunięcie pliku z folderu /uploads
    // Na razie pomijamy ten krok, aby skupić się na logice.
    // W wersji produkcyjnej jest to konieczne, aby nie zostawiać "śmieci" na serwerze.

    info!("Usunięto zdjęcie o ID: {}", photo_id);

    Ok(Redirect::to(&format!("/panel/galleries/{}", gallery_id)))
}
