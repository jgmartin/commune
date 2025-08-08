//! Prelude module for the Commune project.
//!
//! This module re-exports commonly used items from various parts of the crate,
//! as well as from external dependencies. Using `use mcp_commune::prelude::*;`
//! will import these items into scope, making it easier to work with the Commune library.

// Re-export items from the crate's modules
pub use crate::{client::*, error::*, peer::*, server::*, tool::{async_fn_executor, sync_fn_executor}};

// Re-export items from external dependencies
pub use mcp_sdk_rs::types::LoggingLevel;

/// The prelude module.
///
/// This module is intended to be glob-imported (`use mcp_commune::prelude::*;`) to bring
/// commonly used items into scope. It includes re-exports from various parts of the
/// Commune crate as well as selected items from external dependencies.
///
/// # Example
///
/// ```
/// use mcp_commune::prelude::*;
///
/// // Now you can use items from client, error, peer, and server modules
/// // as well as LoggingLevel from mcp_sdk_rs without additional imports.
/// ```
///
/// # Contents
///
/// - All items from the `client` module
/// - All items from the `error` module
/// - All items from the `peer` module
/// - All items from the `server` module
/// - `LoggingLevel` from `mcp_sdk_rs::types`
///
/// By using this prelude, you can reduce the number of explicit imports needed
/// when working with the Commune library, leading to cleaner and more concise code.
pub use super::*;
