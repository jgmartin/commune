# Commune

Commune is a Rust library designed to support the development of discoverable networks of AI agents. It serves as a thin wrapper over the [mcp_rust_sdk](https://github.com/jgmartin/mcp-rust-sdk), providing enhanced functionality for peer discovery and resource utilization within [Model Context Protocol (MCP)](https://spec.modelcontextprotocol.io/specification/2024-11-05/) networks.

## Features

- Discover and maintain a list of peer MCP servers
- Utilize available tools, prompts, and resources from peer servers
- Support for WebSocket communication (with and without TLS encryption)
- Type conversion for AWS, OpenAI, and other inference APIs, simplifying their usage

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
commune = "0.1.0"
```

## Usage

```rust
use commune::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let peer = PeerBuilder::new()
        .with_name("everything".to_string())
        .with_url("ws://localhost:8780".to_string())
        .with_description("various example resources".to_string())
        .build()
        .await?;

    let commune_client = ClientBuilder::new()
        .with_name("zdi-commune".to_string())
        .with_peers(vec![peer])
        .build()
        .await?;

    println!("Calling all_tools()");
    let peer_tools = commune_client.all_tools().await?;
    println!("Available tools: {:?}", peer_tools);

    Ok(())
}
```

This example demonstrates:
1. Creating a new peer using `PeerBuilder`
2. Creating a Commune client using `ClientBuilder`
3. Adding the peer to the client
4. Retrieving all available tools from the list of peers

## Type Conversion

Commune provides convenient type conversion implementations for various inference APIs, including:

- AWS Bedrock
- OpenAI (coming soon)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the [MIT License](LICENSE).
