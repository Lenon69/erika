// src/middleware.rs
use crate::{app_state::AppState, errors::AppError, models::erika::Erika};
use axum::{
    body::Body,
    extract::{FromRequestParts, State},
    http::Request,
    middleware::Next,
    response::Response,
};

use tower_sessions::Session;
use uuid::Uuid;

pub async fn require_admin(
    State(state): State<AppState>,
    request: Request<Body>, // <-- Zmiana na konkretny typ Body
    next: Next,
) -> Result<Response, AppError> {
    let (mut parts, body) = request.into_parts();
    let session = Session::from_request_parts(&mut parts, &state)
        .await
        .map_err(|_| AppError::Unauthorized)?;

    let erika_id = session
        .get::<Uuid>("erika_id")
        .await
        .unwrap_or(None)
        .ok_or(AppError::Unauthorized)?;

    let erika = Erika::find_by_id_for_auth(erika_id, &state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if erika.role == "Admin" {
        let request = Request::from_parts(parts, body);
        Ok(next.run(request).await)
    } else {
        Err(AppError::Unauthorized)
    }
}
