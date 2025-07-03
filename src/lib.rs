use std::borrow::Cow;
use std::collections::HashMap;

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Serialize, Deserialize};

/// A `ServiceError` represents a specific error within the software.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceError<'a> {
    /// An internal error code that represents a specific error within the
    /// system.
    pub code: u32,
    /// A capitalized error name that represents the error type.
    #[serde(borrow)]
    pub name: Cow<'a, str>,
    /// The respective HTTP status code that should be returned to the client.
    #[serde(skip)]
    pub http_status: u16,
    /// A human-readable error message that describes the error in more detail.
    #[serde(borrow)]
    pub message: Cow<'a, str>,
    /// Arguments for message formatting
    #[serde(skip)]
    pub arguments: Vec<String>,
    /// Optional parameters as key-value pairs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<HashMap<String, serde_json::Value>>,
}

impl<'a> ServiceError<'a> {
    /// Create a new [`ServiceError`] instance.
    pub const fn new(code: u32, name: &'a str, status: u16, message: &'a str) -> Self {
        Self {
            code,
            name: Cow::Borrowed(name),
            http_status: status,
            message: Cow::Borrowed(message),
            arguments: Vec::new(),
            parameters: None,
        }
    }

    /// Add an argument for message formatting.
    pub fn bind(mut self, value: impl ToString) -> Self {
        self.arguments.push(value.to_string());
        self
    }

    /// Add an optional parameter.
    pub fn parameter(mut self, key: impl ToString, value: impl Serialize) -> Self {
        let parameters = self.parameters.get_or_insert_with(HashMap::new);
        if let Ok(json_value) = serde_json::to_value(value) {
            parameters.insert(key.to_string(), json_value);
        }
        self
    }

    /// Format the message with provided arguments.
    fn format_message(&self) -> String {
        let mut formatted = self.message.to_string();
        for (i, arg) in self.arguments.iter().enumerate() {
            let placeholder = format!("{{{i}}}");
            formatted = formatted.replace(&placeholder, arg);
        }
        formatted
    }
}

impl<'a> IntoResponse for ServiceError<'a> {
    fn into_response(self) -> Response {
        // try to convert the given status code to a `StatusCode` instance, if it fails,
        // we will default to `INTERNAL_SERVER_ERROR`.
        let status_code =
            StatusCode::from_u16(self.http_status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let response_body = ResponseBody {
            code: self.code,
            name: self.name.clone(),
            message: self.format_message(),
            parameters: self.parameters.clone(),
        };

        (status_code, Json(response_body)).into_response()
    }
}

#[derive(Debug, Clone, Serialize)]
struct ResponseBody<'a> {
    code: u32,
    #[serde(borrow)]
    name: Cow<'a, str>,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<HashMap<String, serde_json::Value>>,
}
