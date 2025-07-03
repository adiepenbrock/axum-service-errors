# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust crate called `axum-service-errors` that provides structured error responses for Axum web applications. The crate defines a `ServiceError` type that implements `IntoResponse` for consistent JSON error responses.

## Core Architecture

- **Single module crate**: All code is in `src/lib.rs`
- **ServiceError struct**: Main error type with fields for error code, name, HTTP status, and message
- **Lifetime management**: Uses `Cow<'a, str>` for zero-copy string handling
- **Axum integration**: Implements `IntoResponse` trait for direct use in Axum handlers

## Common Commands

```bash
# Build the project
cargo build

# Run tests
cargo test

# Check code without building
cargo +nightly check

# Format code
cargo +nightly fmt

# Run clippy linter
cargo +nightly clippy

# Build documentation
cargo doc --open

# Publish to crates.io (when ready)
cargo publish
```

## Key Implementation Details

The `ServiceError` struct in `src/lib.rs:10` is the main component:
- Uses `const fn new()` constructor for compile-time error definitions
- Automatically converts HTTP status codes, defaulting to 500 on invalid codes
- Serializes to JSON via the `IntoResponse` implementation