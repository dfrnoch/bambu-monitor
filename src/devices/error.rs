use super::*;

pub(super) enum ResponseJsonOrStatus<T> {
    Json(T),
    Status(StatusCode),
}

impl<T> IntoResponse for ResponseJsonOrStatus<T>
where
    T: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Json(value) => Json(value).into_response(),
            Self::Status(status) => status.into_response(),
        }
    }
}

#[derive(Debug)]
pub(super) enum AppError {
    Io(std::io::Error),
    Json(serde_json::Error),
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let message = match self {
            Self::Io(error) => error.to_string(),
            Self::Json(error) => error.to_string(),
        };

        (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
    }
}
