use axum::{response::IntoResponse, Json};
use http::StatusCode;
use opentelemetry::trace::Status;
use serde::Serialize;
use thiserror::Error;
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Room not found: {0}")]
    RoomNotFound(String),

    #[error("User not authorized: {0}")]
    Unauthorized(String),

    #[error("Room already exists: {0}")]
    RoomAlreadyExists(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Internal server error: {0}")]
    InternalError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_code, details) = match &self {
            AppError::RoomNotFound(message) => (StatusCode::NOT_FOUND, "ROOM_NOT_FOUND", message),
            AppError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", message),
            AppError::RoomAlreadyExists(message) => {
                (StatusCode::CONFLICT, "ROOM_ALREADY_EXISTS", message)
            }
            AppError::ConnectionError(message) => {
                (StatusCode::BAD_REQUEST, "CONNECTION_ERROR", message)
            }
            AppError::InvalidRequest(message) => {
                (StatusCode::BAD_REQUEST, "INVALID_REQUEST", message)
            }
            AppError::InternalError(message) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", message)
            }
            AppError::DatabaseError(message) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", message)
            }
        };

        let body = Json(ErrorResponse {
            error: self.to_string(),
            code: error_code.to_string(),
            details: details.to_owned(),
        });

        let span = tracing::Span::current();

        // 에러 정보를 span에 기록
        span.record("error", true);
        span.record("error.type", std::any::type_name::<Self>());
        span.record("error.message", self.to_string());

        // 스택 트레이스 캡처 (백트레이스 사용)
        let backtrace = std::backtrace::Backtrace::capture();
        span.record("error.stack", format!("{:?}", backtrace));

        // OpenTelemetry status 설정
        span.set_status(Status::Error {
            description: self.to_string().into(),
        });

        tracing::error!(
            error.type = std::any::type_name::<Self>(),
            error.message = %self,
            error.stack = %backtrace,
            http.status_code = %status.as_u16(),
            "Request failed"
        );

        (status, body).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    code: String,
    details: String,
}
