// src/handlers/gallery_handlers.rs

use super::layout;
use crate::models::gallery::GalleryCategory;
use crate::models::photo::Photo;
use crate::{app_state::AppState, errors::AppError, models::gallery::Gallery};
use axum::extract::{Multipart, Path as AxumPath};
use axum::response::IntoResponse;
use axum::{
    Form,
    extract::State,
    response::{Html, Redirect},
};
use bigdecimal::BigDecimal;
use serde::Deserialize;
use serde::de::Error as _;
use std::path::Path;
use std::str::FromStr;
use strum::IntoEnumIterator;
use tokio::fs;
use tower_sessions::Session;
use tracing::{info, warn};
use uuid::Uuid;
use axum::http::header::HeaderName;

#[derive(Deserialize)]
pub struct CreateGalleryPayload {
    name: GalleryCategory,
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
                        // Zamieniamy pole tekstowe na listę rozwijaną
                        select name="name" required
                               class="flex-grow px-3 py-2 bg-gray-700 border border-gray-600 rounded-md text-white focus:outline-none focus:ring-2 focus:ring-blue-500" {
                            option value="" disabled selected { "Wybierz kategorię..." }
                            @for category in GalleryCategory::iter() {
                                option value=(category.to_string()) { (category.to_string()) }
                            }
                        }
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

    Gallery::create(erika_id, payload.name, &state.db)
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
    let erika_id = session
        .get::<Uuid>("erika_id")
        .await
        .unwrap_or(None)
        .ok_or(AppError::Unauthorized)?;

    // Pobieramy dane galerii, żeby wypełnić formularz
    let gallery = Gallery::find_by_id_and_erika_id(gallery_id, erika_id, &state.db) // <-- ZMIANA
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or(AppError::Unauthorized)?; // Zwróć błąd, jeśli galeria nie należy do tej Eriki

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
                div id="photo-grid" class="grid grid-cols-2 md:grid-cols-4 gap-4" {
                    @for photo in photos {
                        (maud::PreEscaped(render_photo_partial(gallery_id, &photo)))
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
    pub name: String,
    pub description: String,
    #[serde(deserialize_with = "empty_string_as_none")]
    pub price_pln: Option<BigDecimal>,
}

// NOWY HANDLER: Aktualizuje dane galerii

pub async fn update_gallery(
    AxumPath(gallery_id): AxumPath<Uuid>,
    session: Session,
    State(state): State<AppState>,
    Form(payload): Form<UpdateGalleryPayload>,
) -> Result<Redirect, AppError> {
    let erika_id = session
        .get::<Uuid>("erika_id")
        .await
        .unwrap_or(None)
        .ok_or(AppError::Unauthorized)?;

    Gallery::find_by_id_and_erika_id(gallery_id, erika_id, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or(AppError::Unauthorized)?;

    // --- POPRAWKA TUTAJ ---
    // 1. Parsujemy String z formularza na nasz enum GalleryCategory
    let category =
        GalleryCategory::from_str(&payload.name).map_err(|_| AppError::InternalServerError)?; // Obsłuż błąd, jeśli nazwa jest nieprawidłowa

    // 2. Przekazujemy sparsowaną kategorię do modelu
    Gallery::update_details(
        gallery_id,
        category, // <-- Przekazujemy poprawny typ
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
) -> Result<impl IntoResponse, AppError> {
    let erika_id = session
        .get::<Uuid>("erika_id")
        .await
        .unwrap_or(None)
        .ok_or(AppError::Unauthorized)?;

    // 1. Sprawdź, czy galeria należy do zalogowanej Eriki
    let gallery = Gallery::find_by_id_and_erika_id(gallery_id, erika_id, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or(AppError::Unauthorized)?;

    // 2. Znajdź zdjęcie, aby uzyskać jego URL
    let photo = Photo::find_by_id(photo_id, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or(AppError::NotFound)?;

    // Upewnij się, że zdjęcie jest w odpowiedniej galerii
    if photo.gallery_id != gallery.id {
        return Err(AppError::Unauthorized);
    }

    // 3. Usuń plik z dysku
    // Usuwamy wiodący '/' z URL, aby otrzymać ścieżkę do pliku
    let file_path = photo.file_url.strip_prefix('/').unwrap_or(&photo.file_url);
    fs::remove_file(file_path).await.map_err(|e| {
        warn!("Nie udało się usunąć pliku {}: {}", file_path, e);
        AppError::InternalServerError
    })?;

    // 4. Usuń wpis z bazy danych
    Photo::delete(photo_id, &state.db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    info!("Usunięto zdjęcie o ID: {}", photo_id);
    
    // Po usunięciu, pobierz odświeżoną listę zdjęć
    let photos = Photo::find_by_gallery_id(gallery_id, &state.db).await?;
    let updated_grid = render_photos_grid(gallery_id, &photos);

    // --- POPRAWKA TUTAJ ---
    // Zwracamy odpowiedź z nagłówkiem, który wywoła nasze zdarzenie `closeModal`
    Ok((
        [(axum::http::header::HeaderName::from_static("hx-trigger"), "closeModal")],
        Html(updated_grid),
    ))
}

// NOWY HANDLER: Zwraca fragment HTML z potwierdzeniem usunięcia
pub async fn confirm_delete_photo(
    AxumPath((gallery_id, photo_id)): AxumPath<(Uuid, Uuid)>,
) -> Result<Html<String>, AppError> {
    let content = maud::html! {
        div class="text-center" {
            p class="text-white mb-4 text-lg" { "Czy na pewno chcesz usunąć to zdjęcie?" }
            div class="flex justify-center gap-4" {
                button hx-post=(format!("/panel/galleries/{}/photo/{}/delete", gallery_id, photo_id))
                       hx-target="#photo-grid"
                       hx-swap="innerHTML"
                       class="bg-red-600 hover:bg-red-700 text-white font-bold py-2 px-4 rounded-md" {
                    "Tak, usuń"
                }
                // --- POPRAWKA TUTAJ ---
                // Używamy `@click` z Alpine.js do zamknięcia modala
                button type="button" "@click"="modalOpen = false"
                       class="bg-gray-600 hover:bg-gray-700 text-white font-bold py-2 px-4 rounded-md" {
                    "Anuluj"
                }
            }
        }
    };
    Ok(Html(content.into_string()))
}

// Potrzebujemy też handlera, który zwróci HTML dla pojedynczego zdjęcia (do anulowania)
pub async fn get_photo_partial(
    AxumPath((gallery_id, photo_id)): AxumPath<(Uuid, Uuid)>,
    State(state): State<AppState>,
) -> Result<Html<String>, AppError> {
    // Tutaj normalnie pobralibyśmy dane zdjęcia z bazy, ale na razie uprośćmy
    // i załóżmy, że potrzebujemy tylko URL-a, który już mamy w innej funkcji.
    // W przyszłości można to zoptymalizować.
    let photo = crate::models::photo::Photo::find_by_id(photo_id, &state.db)
        .await // <-- POTRZEBUJEMY TEJ FUNKCJI
        .map_err(|_| AppError::InternalServerError)?
        .ok_or(AppError::NotFound)?;

    Ok(Html(render_photo_partial(gallery_id, &photo)))
}

/// Renderuje fragment HTML dla JEDNEGO zdjęcia.
fn render_photo_partial(gallery_id: Uuid, photo: &Photo) -> String {
    maud::html! {
        // Kontener dla zdjęcia jest teraz celem dla HTMX
        div class="photo-container bg-gray-800 rounded-lg overflow-hidden shadow-lg relative group" {
            img src=(photo.file_url) alt="Zdjęcie z galerii" class="w-full h-48 object-cover";

            // Nakładka jest teraz JEDNYM wielkim, klikalnym przyciskiem dla HTMX.
            // Po załadowaniu treści do modala, aktywuje go (`x-on:htmx:after-swap`).
            div hx-get=(format!("/panel/photo/delete-confirm/{}/{}", gallery_id, photo.id))
                hx-target="#modal-content"
                hx-swap="innerHTML"
                x-on:htmx:after-swap="modalOpen = true"
                class="absolute inset-0 z-10 bg-black/60 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer" {
                
                // To jest tylko wizualna etykieta, a nie faktyczny przycisk.
                span class="bg-red-600 text-white font-bold py-2 px-4 rounded-md pointer-events-none" {
                    "Usuń"
                }
            }
        }
    }.into_string()
}

/// Renderuje całą siatkę zdjęć.
fn render_photos_grid(gallery_id: Uuid, photos: &[Photo]) -> String {
    maud::html! {
        div id="photo-grid" class="grid grid-cols-2 md:grid-cols-4 gap-4" {
            @if photos.is_empty() {
                p class="text-gray-400 col-span-full" { "Brak zdjęć w tej galerii." }
            } @else {
                @for photo in photos {
                    (maud::PreEscaped(render_photo_partial(gallery_id, photo)))
                }
            }
        }
    }.into_string()
}

// NOWY HANDLER: Zwraca tylko fragment HTML z siatką zdjęć
async fn get_photos_grid_partial(
    gallery_id: Uuid,
    db: &sqlx::PgPool,
) -> Result<Html<String>, AppError> {
    let photos = Photo::find_by_gallery_id(gallery_id, db)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    let content = maud::html! {
        @if photos.is_empty() {
            p class="text-gray-400" { "Brak zdjęć w tej galerii." }
        } @else {
            div id="photo-grid" class="grid grid-cols-2 md:grid-cols-4 gap-4" {
                @for photo in photos {
                    (maud::PreEscaped(render_photo_partial(gallery_id, &photo)))
                }
            }
        }
    };
    Ok(Html(content.into_string()))
}

// --- NOWA FUNKCJA POMOCNICZA ---
// Ta funkcja "uczy" serde, jak traktować puste stringi jako None dla liczb.
fn empty_string_as_none<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        s.parse::<T>().map(Some).map_err(D::Error::custom)
    }
}
