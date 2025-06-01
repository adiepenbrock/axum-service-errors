use std::borrow::Cow;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// A `ServiceError` represents a specific error within the software.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServiceError<'a> {
    /// An internal error code that represents a specific error within the
    /// system.
    pub code: u32,
    /// A capitalized error name that represents the error type.
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    /// The respective HTTP status code that should be returned to the client.
    pub http_status: u16,
    /// A human-readable error message that describes the error in more detail.
    #[serde(borrow)]
    pub message: Cow<'a, str>,
}

impl<'a> ServiceError<'a> {
    /// Create a new [`ServiceError`] instance.
    pub const fn new(code: u32, name: &'a str, status: u16, message: &'a str) -> Self {
        Self {
            code,
            name: Cow::Borrowed(name),
            http_status: status,
            message: Cow::Borrowed(message),
        }
    }
}

impl<'a> IntoResponse for ServiceError<'a> {
    fn into_response(self) -> Response {
        // try to convert the given status code to a `StatusCode` instance, if it fails,
        // we will default to `INTERNAL_SERVER_ERROR`.
        let status_code =
            StatusCode::from_u16(self.http_status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        (status_code, Json(self)).into_response()
    }
}
