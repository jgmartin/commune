//! # Commune
//!
//! `commune` is a Rust library for building distributed systems and peer-to-peer networks.
//! It provides a set of modules to handle client-server communication, peer management,
//! error handling, and more.
//!
//! ## Modules
//!
//! - `client`: Implements client-side functionality for connecting to servers and peers.
//! - `error`: Defines custom error types used throughout the library.
//! - `peer`: Manages peer connections and interactions in a distributed network.
//! - `prelude`: Provides a convenient way to import commonly used items from the library.
//! - `server`: Implements server-side functionality for handling client connections and requests.

/// Client-side functionality module
pub mod client;

/// Error handling module
pub mod error;

/// Peer management module
pub mod peer;

/// Functionality related to tool calling
pub mod tool;

/// Prelude module for convenient imports
pub mod prelude;

/// Server-side functionality module
pub mod server;
