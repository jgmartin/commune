use derive_more::Display;

/// Error types
#[derive(Debug, Display)]
pub enum Error {
    #[display("internal server error")]
    Internal,
    #[display("unsupported feature")]
    Unsupported,
    #[display("uninitialized client")]
    UninitializedClient,
    #[display("mcp client error")]
    McpClient(String),
    #[display("invalid response")]
    InvalidResponse,
    #[display("serialization error")]
    Serialization,
    #[display("client initialization error")]
    ClientInitialization,
}
impl std::error::Error for Error {}
