use axum::{response::{IntoResponse, Response}, http::StatusCode};

pub enum AppError {
    InternalServerError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Wystąpił wewnętrzny błąd serwera",
            ),
        };
        (status, error_message).into_response()
    }
}
