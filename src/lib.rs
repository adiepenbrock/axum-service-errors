use std::borrow::Cow;
use std::collections::HashMap;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Serialize, Deserialize};

/// A trait for building custom response formats from ServiceError data.
pub trait ResponseBuilder: std::fmt::Debug {
    /// Build a response body and content-type from the error data.
    fn build(&self, error: &ServiceError) -> (String, &'static str);
}

/// A `ServiceError` represents a specific error within the software.
#[derive(Debug, Serialize, Deserialize)]
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
    pub parameters: Option<HashMap<String, String>>,
    /// Custom response builder for formatting output
    #[serde(skip)]
    response_builder: Option<Box<dyn ResponseBuilder>>,
}

impl<'a> Clone for ServiceError<'a> {
    fn clone(&self) -> Self {
        Self {
            code: self.code,
            name: self.name.clone(),
            http_status: self.http_status,
            message: self.message.clone(),
            arguments: self.arguments.clone(),
            parameters: self.parameters.clone(),
            response_builder: None, // Cannot clone trait objects
        }
    }
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
            response_builder: None,
        }
    }

    /// Add an argument for message formatting.
    pub fn bind(mut self, value: impl ToString) -> Self {
        self.arguments.push(value.to_string());
        self
    }

    /// Add an optional parameter.
    pub fn parameter(mut self, key: impl ToString, value: impl ToString) -> Self {
        let parameters = self.parameters.get_or_insert_with(HashMap::new);
        parameters.insert(key.to_string(), value.to_string());
        self
    }

    /// Set a custom response builder for formatting the response.
    pub fn with_response_builder(mut self, builder: impl ResponseBuilder + 'static) -> Self {
        self.response_builder = Some(Box::new(builder));
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
        let status_code =
            StatusCode::from_u16(self.http_status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let (body, content_type) = if let Some(builder) = &self.response_builder {
            builder.build(&self)
        } else {
            // Default plain text format
            let text = if let Some(ref params) = self.parameters {
                format!(
                    "Error {}: {} - {} (Parameters: {:?})",
                    self.code,
                    self.name,
                    self.format_message(),
                    params
                )
            } else {
                format!(
                    "Error {}: {} - {}",
                    self.code,
                    self.name,
                    self.format_message()
                )
            };
            (text, "text/plain")
        };

        (
            status_code,
            [("content-type", content_type)],
            body,
        ).into_response()
    }
}

/// A simple JSON response builder that serializes the ServiceError as JSON.
#[cfg(feature = "json")]
#[derive(Debug, Clone)]
pub struct JsonResponseBuilder;

#[cfg(feature = "json")]
impl JsonResponseBuilder {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(feature = "json")]
impl ResponseBuilder for JsonResponseBuilder {
    fn build(&self, error: &ServiceError) -> (String, &'static str) {
        let response_body = JsonResponseBody {
            code: error.code,
            name: error.name.clone(),
            message: error.format_message(),
            parameters: error.parameters.clone(),
        };
        
        let json = serde_json::to_string(&response_body)
            .unwrap_or_else(|_| format!("{{\"error\":\"Failed to serialize error {}\"}}", error.code));
        
        (json, "application/json")
    }
}

#[cfg(feature = "json")]
#[derive(Debug, Clone, Serialize)]
struct JsonResponseBody<'a> {
    code: u32,
    #[serde(borrow)]
    name: Cow<'a, str>,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parameters: Option<HashMap<String, String>>,
}

/// A simple plain text response builder.
#[derive(Debug, Clone)]
pub struct PlainTextResponseBuilder;

impl PlainTextResponseBuilder {
    pub fn new() -> Self {
        Self
    }
}

impl ResponseBuilder for PlainTextResponseBuilder {
    fn build(&self, error: &ServiceError) -> (String, &'static str) {
        let text = if let Some(ref params) = error.parameters {
            format!(
                "Error {}: {} - {} (Parameters: {:?})",
                error.code,
                error.name,
                error.format_message(),
                params
            )
        } else {
            format!(
                "Error {}: {} - {}",
                error.code,
                error.name,
                error.format_message()
            )
        };
        (text, "text/plain")
    }
}
