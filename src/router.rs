// src/router.rs

use crate::{
    app_state::AppState,
    handlers::{admin_handlers, erika_handlers, gallery_handlers},
    middleware,
};

use axum::{
    Router, // Dodajemy `post`
    middleware as axum_middleware,
    routing::{get, post},
};
use tower_http::services::ServeDir;

pub fn create_router(app_state: AppState) -> Router {
    // Grupujemy ścieżki admina i nakładamy na nie nasz middleware
    let admin_routes = Router::new()
        .route("/", get(admin_handlers::admin_dashboard))
        .route(
            "/erika/{erika_id}",
            get(admin_handlers::show_edit_erika_form).post(admin_handlers::update_erika_by_admin),
        )
        .route(
            "/erika/{erika_id}/approve",
            post(admin_handlers::approve_erika),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            app_state.clone(),
            middleware::require_admin,
        ));

    Router::new()
        .route("/", get(erika_handlers::homepage))
        .route(
            "/register",
            get(erika_handlers::show_register_form).post(erika_handlers::register_erika),
        )
        .route(
            "/login",
            get(erika_handlers::show_login_form).post(erika_handlers::login_erika),
        )
        .route(
            "/panel",
            get(erika_handlers::erika_panel).post(erika_handlers::update_erika_profile),
        )
        .route(
            "/panel/galleries",
            get(gallery_handlers::show_galleries_page).post(gallery_handlers::create_gallery),
        )
        .route(
            "/panel/galleries/{gallery_id}",
            get(gallery_handlers::show_single_gallery_page).post(gallery_handlers::update_gallery),
        )
        .route(
            "/panel/galleries/{gallery_id}/photo/{photo_id}/delete",
            post(gallery_handlers::delete_photo),
        )
        .route(
            "/panel/photo/delete-confirm/{gallery_id}/{photo_id}",
            get(gallery_handlers::confirm_delete_photo),
        )
        .route(
            "/panel/galleries/{gallery_id}/photo/{photo_id}",
            get(gallery_handlers::get_photo_partial),
        )
        .route(
            "/panel/galleries/{gallery_id}/upload",
            post(gallery_handlers::upload_photo),
        )
        .route("/erika/{username}", get(erika_handlers::show_erika_profile))
        .route(
            "/pay/gallery/{gallery_id}",
            get(erika_handlers::initiate_gallery_payment),
        )
        .route("/logout", post(erika_handlers::logout))
        .route("/panel/stream", get(erika_handlers::show_stream_panel))
        .route(
            "/panel/status-toggle",
            post(erika_handlers::toggle_online_status),
        )
        .nest("/admin", admin_routes)
        .nest_service("/uploads", ServeDir::new("uploads"))
        .with_state(app_state)
}
