# axum-service-errors

A Rust crate that provides structured error responses for [Axum](https://github.com/tokio-rs/axum) web applications.

## Features

- **Structured Error Handling**: Define errors with error codes, names, HTTP status codes, and messages
- **Zero-Copy Strings**: Uses `Cow<'a, str>` for efficient string handling
- **Message Formatting**: Support for parameterized messages with argument binding
- **Pluggable Response Builders**: Customize response formats with global defaults or per-error overrides
- **Axum Integration**: Implements `IntoResponse` for seamless use in Axum handlers
- **Serialization Support**: Serde support for error serialization
- **Optional JSON Feature**: Enable JSON responses with the `json` feature flag

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
axum-service-errors = "0.3.3"

# Enable JSON support (optional)
axum-service-errors = { version = "0.3.3", features = ["json"] }
```

## Quick Start

```rust
use axum_service_errors::{ServiceError, JsonResponseBuilder, set_default_response_builder};
use axum::{routing::get, Router};

async fn handler() -> Result<String, ServiceError<'static>> {
    // Return an error that will be automatically converted to an HTTP response
    Err(ServiceError::new(1001, "VALIDATION_ERROR", 400, "Invalid input provided"))
}

#[tokio::main]
async fn main() {
    // Set a global default response builder (optional)
    set_default_response_builder(JsonResponseBuilder::new());
    
    let app = Router::new().route("/", get(handler));
    // ... start server
}
```

## Usage Examples

### Basic Error Creation

```rust
use axum_service_errors::ServiceError;

// Create a basic error
let error = ServiceError::new(1001, "VALIDATION_ERROR", 400, "Invalid input");
```

### Message Formatting with Arguments

```rust
// Error with parameterized message
let error = ServiceError::new(1001, "VALIDATION_ERROR", 400, "Invalid {0} for field {1}")
    .bind("email address")
    .bind("user.email");

// Results in: "Invalid email address for field user.email"
```

### Adding Parameters

```rust
// Add optional parameters for additional context
let error = ServiceError::new(1001, "VALIDATION_ERROR", 400, "Invalid input")
    .parameter("field", "email")
    .parameter("reason", "malformed");
```

### Global Default Response Builder

```rust
use axum_service_errors::{ServiceError, JsonResponseBuilder, set_default_response_builder};

// Set global default at application startup
set_default_response_builder(JsonResponseBuilder::new());

// Now all errors automatically use JSON format
let error = ServiceError::new(1001, "VALIDATION_ERROR", 400, "Invalid input")
    .parameter("field", "email");
// No need to call .with_response_builder() - uses JSON by default!
```

### Per-Error Response Builder Override

```rust
use axum_service_errors::{ServiceError, JsonResponseBuilder, PlainTextResponseBuilder};

// Set JSON as global default
set_default_response_builder(JsonResponseBuilder::new());

// This error will use JSON (global default)
let json_error = ServiceError::new(1001, "VALIDATION_ERROR", 400, "Invalid input");

// This error overrides to use plain text
let text_error = ServiceError::new(1002, "SYSTEM_ERROR", 500, "System failure")
    .with_response_builder(PlainTextResponseBuilder::new());
```

### Custom Response Builder

```rust
use axum_service_errors::{ServiceError, ResponseBuilder, set_default_response_builder};

#[derive(Debug)]
struct CustomBuilder;

impl ResponseBuilder for CustomBuilder {
    fn build(&self, error: &ServiceError) -> (String, &'static str) {
        let body = format!("Custom error: {}", error.message);
        (body, "text/plain")
    }
}

// Set as global default
set_default_response_builder(CustomBuilder);

// Or use per-error
let error = ServiceError::new(1001, "CUSTOM_ERROR", 500, "Something went wrong")
    .with_response_builder(CustomBuilder);
```

## Features

### Default Features

- Plain text response formatting (fallback when no global default is set)
- Basic error structure with code, name, status, and message
- Message formatting with arguments
- Optional parameters support
- Global default response builder configuration

### JSON Feature

Enable with `features = ["json"]` in your `Cargo.toml`:

```toml
[dependencies]
axum-service-errors = { version = "0.3.3", features = ["json"] }
```

Provides:
- `JsonResponseBuilder` for JSON-formatted error responses
- Automatic JSON serialization of error data
- Can be set as global default with `set_default_response_builder(JsonResponseBuilder::new())`

## Error Structure

The `ServiceError` struct contains:

- `code`: Internal error code (u32)
- `name`: Error type name (e.g., "VALIDATION_ERROR")
- `http_status`: HTTP status code for the response
- `message`: Human-readable error message
- `arguments`: Values for message formatting (not serialized)
- `parameters`: Optional key-value pairs for additional context
- `response_builder`: Optional custom response formatter (not serialized)

## Development

### Building

```bash
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with JSON feature
cargo test --features json
```

### Formatting and Linting

```bash
cargo fmt
cargo clippy
```

## License

This project is licensed under the MIT License.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

