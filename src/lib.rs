use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::OnceLock;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

/// A parameter value that can be nested and supports various data types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParameterValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<ParameterValue>),
    Object(HashMap<String, ParameterValue>),
    Null,
}

impl From<String> for ParameterValue {
    fn from(value: String) -> Self {
        ParameterValue::String(value)
    }
}

impl From<&str> for ParameterValue {
    fn from(value: &str) -> Self {
        ParameterValue::String(value.to_string())
    }
}

impl From<i32> for ParameterValue {
    fn from(value: i32) -> Self {
        ParameterValue::Integer(value as i64)
    }
}

impl From<i64> for ParameterValue {
    fn from(value: i64) -> Self {
        ParameterValue::Integer(value)
    }
}

impl From<f32> for ParameterValue {
    fn from(value: f32) -> Self {
        ParameterValue::Float(value as f64)
    }
}

impl From<f64> for ParameterValue {
    fn from(value: f64) -> Self {
        ParameterValue::Float(value)
    }
}

impl From<bool> for ParameterValue {
    fn from(value: bool) -> Self {
        ParameterValue::Boolean(value)
    }
}

impl From<Vec<ParameterValue>> for ParameterValue {
    fn from(value: Vec<ParameterValue>) -> Self {
        ParameterValue::Array(value)
    }
}

impl From<HashMap<String, ParameterValue>> for ParameterValue {
    fn from(value: HashMap<String, ParameterValue>) -> Self {
        ParameterValue::Object(value)
    }
}

impl From<Vec<String>> for ParameterValue {
    fn from(value: Vec<String>) -> Self {
        ParameterValue::Array(value.into_iter().map(|v| v.into()).collect())
    }
}

impl From<Vec<&str>> for ParameterValue {
    fn from(value: Vec<&str>) -> Self {
        ParameterValue::Array(value.into_iter().map(|v| v.into()).collect())
    }
}

impl From<Vec<i32>> for ParameterValue {
    fn from(value: Vec<i32>) -> Self {
        ParameterValue::Array(value.into_iter().map(|v| v.into()).collect())
    }
}

impl From<Vec<i64>> for ParameterValue {
    fn from(value: Vec<i64>) -> Self {
        ParameterValue::Array(value.into_iter().map(|v| v.into()).collect())
    }
}

impl From<Vec<f64>> for ParameterValue {
    fn from(value: Vec<f64>) -> Self {
        ParameterValue::Array(value.into_iter().map(|v| v.into()).collect())
    }
}

impl From<Vec<bool>> for ParameterValue {
    fn from(value: Vec<bool>) -> Self {
        ParameterValue::Array(value.into_iter().map(|v| v.into()).collect())
    }
}

impl Display for ParameterValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterValue::String(s) => write!(f, "{}", s),
            ParameterValue::Integer(i) => write!(f, "{}", i),
            ParameterValue::Float(float) => write!(f, "{}", float),
            ParameterValue::Boolean(b) => write!(f, "{}", b),
            ParameterValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            ParameterValue::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, value)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            }
            ParameterValue::Null => write!(f, "null"),
        }
    }
}

impl ParameterValue {
    /// Create a new array parameter value.
    pub fn array(items: Vec<impl Into<ParameterValue>>) -> Self {
        ParameterValue::Array(items.into_iter().map(|v| v.into()).collect())
    }

    /// Create a new object parameter value.
    pub fn object(map: impl Into<HashMap<String, ParameterValue>>) -> Self {
        ParameterValue::Object(map.into())
    }
}

/// A trait for building custom response formats from ServiceError data.
pub trait ResponseBuilder: std::fmt::Debug + Send + Sync {
    /// Build a response body and content-type from the error data.
    fn build(&self, error: &ServiceError) -> (String, &'static str);
}

/// Global default response builder storage.
static DEFAULT_RESPONSE_BUILDER: OnceLock<Box<dyn ResponseBuilder>> = OnceLock::new();

/// Set the global default response builder for all ServiceError instances.
/// This should be called once at application startup.
pub fn set_default_response_builder(builder: impl ResponseBuilder + 'static) {
    DEFAULT_RESPONSE_BUILDER.set(Box::new(builder)).ok();
}

/// Get the global default response builder, if one has been set.
fn get_default_response_builder() -> Option<&'static Box<dyn ResponseBuilder>> {
    DEFAULT_RESPONSE_BUILDER.get()
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
    pub parameters: Option<HashMap<String, ParameterValue>>,
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
    pub fn parameter(mut self, key: impl ToString, value: impl Into<ParameterValue>) -> Self {
        let parameters = self.parameters.get_or_insert_with(HashMap::new);
        parameters.insert(key.to_string(), value.into());
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
            // Use instance-specific builder
            builder.build(&self)
        } else if let Some(default_builder) = get_default_response_builder() {
            // Use global default builder
            default_builder.build(&self)
        } else {
            // Fallback to plain text format
            let text = if let Some(ref params) = self.parameters {
                let param_display: Vec<String> = params
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                format!(
                    "Error {}: {} - {} (Parameters: {{{}}})",
                    self.code,
                    self.name,
                    self.format_message(),
                    param_display.join(", ")
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

        (status_code, [("content-type", content_type)], body).into_response()
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

        let json = serde_json::to_string(&response_body).unwrap_or_else(|_| {
            format!("{{\"error\":\"Failed to serialize error {}\"}}", error.code)
        });

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
    parameters: Option<HashMap<String, ParameterValue>>,
}

/// A simple plain text response builder.
#[derive(Debug, Clone)]
pub struct PlainTextResponseBuilder;

impl Default for PlainTextResponseBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl PlainTextResponseBuilder {
    pub fn new() -> Self {
        Self
    }
}

impl ResponseBuilder for PlainTextResponseBuilder {
    fn build(&self, error: &ServiceError) -> (String, &'static str) {
        let text = if let Some(ref params) = error.parameters {
            let param_display: Vec<String> = params
                .iter()
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect();
            format!(
                "Error {}: {} - {} (Parameters: {{{}}})",
                error.code,
                error.name,
                error.format_message(),
                param_display.join(", ")
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
