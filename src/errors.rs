use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive()]
pub enum AppError {
    InternalServerError,
    Unauthorized,
    NotFound,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Wystąpił wewnętrzny błąd serwera",
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "Brak autoryzacji. Musisz być zalogowany.",
            ),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Nie znaleziono zasobu"),
        };
        (status, error_message).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(_err: sqlx::Error) -> Self {
        // Na razie każdy błąd z bazy traktujemy jako wewnętrzny błąd serwera.
        AppError::InternalServerError
    }
}
