use axum::response::IntoResponse;
use axum_service_errors::{PlainTextResponseBuilder, ResponseBuilder, ServiceError};

#[cfg(feature = "json")]
use axum_service_errors::JsonResponseBuilder;

#[test]
fn test_default_plain_text_response() {
    let error = ServiceError::new(1001, "VALIDATION_ERROR", 400, "Invalid input");

    // Test IntoResponse conversion with default plain text
    let response = error.into_response();
    assert_eq!(response.status(), 400);
}

#[test]
fn test_plain_text_response_builder() {
    let error = ServiceError::new(1001, "VALIDATION_ERROR", 400, "Invalid input");
    let builder = PlainTextResponseBuilder::new();

    let (body, content_type) = builder.build(&error);
    assert_eq!(content_type, "text/plain");
    assert!(body.contains("Error 1001"));
    assert!(body.contains("VALIDATION_ERROR"));
    assert!(body.contains("Invalid input"));
}

#[test]
fn test_plain_text_with_parameters() {
    let error = ServiceError::new(1001, "VALIDATION_ERROR", 400, "Invalid input")
        .parameter("field", "email")
        .parameter("reason", "malformed");

    let builder = PlainTextResponseBuilder::new();
    let (body, content_type) = builder.build(&error);

    assert_eq!(content_type, "text/plain");
    assert!(body.contains("Parameters:"));
}

#[test]
fn test_plain_text_with_arguments() {
    let error = ServiceError::new(1001, "VALIDATION_ERROR", 400, "Invalid input for field {0}")
        .bind("email");

    let builder = PlainTextResponseBuilder::new();
    let (body, _) = builder.build(&error);

    assert!(body.contains("Invalid input for field email"));
}

#[test]
fn test_with_response_builder() {
    let error = ServiceError::new(1001, "VALIDATION_ERROR", 400, "Invalid input")
        .with_response_builder(PlainTextResponseBuilder::new());

    let response = error.into_response();
    assert_eq!(response.status(), 400);
}

#[cfg(feature = "json")]
#[test]
fn test_json_response_builder() {
    let error = ServiceError::new(1001, "VALIDATION_ERROR", 400, "Invalid input")
        .parameter("field", "email");

    let builder = JsonResponseBuilder::new();
    let (body, content_type) = builder.build(&error);

    assert_eq!(content_type, "application/json");
    assert!(body.contains("\"code\":1001"));
    assert!(body.contains("\"name\":\"VALIDATION_ERROR\""));
    assert!(body.contains("\"message\":\"Invalid input\""));
    assert!(body.contains("\"parameters\""));
}

#[cfg(feature = "json")]
#[test]
fn test_json_response_with_formatting() {
    let error = ServiceError::new(1001, "VALIDATION_ERROR", 400, "Invalid {0} for {1}")
        .bind("value")
        .bind("field");

    let builder = JsonResponseBuilder::new();
    let (body, _) = builder.build(&error);

    assert!(body.contains("Invalid value for field"));
}

#[test]
fn test_service_error_serialization() {
    let error = ServiceError::new(1001, "VALIDATION_ERROR", 400, "Invalid input")
        .parameter("field", "email")
        .bind("test"); // This should not appear in serialization

    let serialized = serde_json::to_string(&error).unwrap();

    // Should contain serializable fields
    assert!(serialized.contains("\"code\":1001"));
    assert!(serialized.contains("\"name\":\"VALIDATION_ERROR\""));
    assert!(serialized.contains("\"message\":\"Invalid input\""));
    assert!(serialized.contains("\"parameters\""));

    // Should NOT contain skipped fields
    assert!(!serialized.contains("http_status"));
    assert!(!serialized.contains("arguments"));
    assert!(!serialized.contains("response_builder"));
}

#[test]
fn test_clone_loses_response_builder() {
    let error = ServiceError::new(1001, "VALIDATION_ERROR", 400, "Invalid input")
        .with_response_builder(PlainTextResponseBuilder::new());

    let cloned = error.clone();

    // Original should work with custom builder
    let response1 = error.into_response();

    // Clone should use default behavior (no custom builder)
    let response2 = cloned.into_response();

    assert_eq!(response1.status(), response2.status());
}
