use derive_more::{Display, Error as OtherError};

/// Error types
#[derive(Debug, Display, OtherError)]
pub enum Error {
    #[display("internal server error")]
    Internal,
    #[display("unsupported feature")]
    Unsupported,
    #[display("uninitialized client")]
    UninitializedClient,
    #[display("mcp client error")]
    McpClient,
    #[display("invalid response")]
    InvalidResponse,
    #[display("serialization error")]
    Serialization,
    #[display("client initialization error")]
    ClientInitialization,
}
