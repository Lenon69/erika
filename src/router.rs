// src/router.rs

use crate::{app_state::AppState, handlers::erika_handlers};
use axum::{
    Router,
    routing::{get, post}, // Dodajemy `post`
};

pub fn create_router(app_state: AppState) -> Router {
    Router::new()
        .route("/", get(erika_handlers::show_register_form))
        .route("/register", post(erika_handlers::register_erika))
        .route(
            "/login",
            get(erika_handlers::show_login_form).post(erika_handlers::login_erika),
        )
        .route(
            "/panel",
            get(erika_handlers::erika_panel).post(erika_handlers::update_erika_profile),
        )
        .with_state(app_state)
}
