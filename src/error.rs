//! # Error Module
//!
//! This module defines the custom error types used throughout the Commune library.
//! It provides a centralized way to handle and represent various error conditions
//! that may occur during the operation of the library.

use derive_more::Display;

/// Represents the various error types that can occur in the Commune library.
///
/// This enum implements both `Debug` and `Display` traits for better error reporting.
#[derive(Debug, Display)]
pub enum Error {
    /// Represents an internal server error.
    #[display("internal server error")]
    Internal,

    /// Indicates that a requested feature is not supported.
    #[display("unsupported feature")]
    Unsupported,

    /// Occurs when trying to use an uninitialized client.
    #[display("uninitialized client")]
    UninitializedClient,

    /// Represents errors originating from the MCP client.
    #[display("mcp client error")]
    McpClient(String),

    /// Indicates that a response received is invalid or cannot be processed.
    #[display("invalid response")]
    InvalidResponse,

    /// Represents errors during serialization or deserialization processes.
    #[display("serialization error")]
    Serialization,

    /// Occurs when there's an error initializing a client.
    #[display("client initialization error")]
    ClientInitialization,
}

/// Implements the standard error trait for our custom Error enum.
///
/// This allows our Error type to be used in contexts where a standard error is expected.
impl std::error::Error for Error {}