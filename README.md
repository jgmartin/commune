# Commune

Commune is a Rust library designed to support the development of discoverable networks of AI agents. It serves as a wrapper over [mcp_sdk_rs](https://github.com/jgmartin/mcp-sdk-rs), providing enhanced functionality for peer discovery and resource utilization within [Model Context Protocol (MCP)](https://spec.modelcontextprotocol.io/specification/2024-11-05/) networks.

## Features

- Discover and maintain a list of peer MCP servers
- Utilize available tools, prompts, and resources from peer servers
- Support for WebSocket communication (with and without TLS encryption)
- Type conversion for AWS, OpenAI, and other inference APIs, simplifying their usage

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
commune = { package = "mcp-commune", version = "0.1.2" }
```

## Usage

```rust
use commune::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a peer for each MCP server
    let peer1 = PeerBuilder::new()
        .with_name("everything".to_string())
        .with_url("ws://localhost:8780".to_string())
        .with_description("various example resources".to_string())
        .build()
        .await?;
    let peer2 = PeerBuilder::new()
        .with_name("memory".to_string())
        .with_url("ws://localhost:8781".to_string())
        .with_description("memory based on a knowledge graph".to_string())
        .build()
        .await?;

    // Optionally create a commune client
    let commune_client = ClientBuilder::new()
        .with_peers(vec![peer1.clone(), peer2])
        .build()
        .await?;

    // Use the client to aggregate resources
    // Get all tools
    let peer_tools = commune_client.all_tools().await?;
    log::info!("found {} tools", peer_tools.len());

    // Get all resources
    let peer_resources = commune_client.all_resources().await?;
    log::info!("found {} resources!", peer_resources.len());

    // Get all prompts
    let peer_prompts = commune_client.all_prompts().await?;
    log::info!("found {} prompts!", peer_prompts.len());

    // Use peers to utilize them, subscribe to notifications etc
    // Subscribe to resource updates
    peer1.subscribe("test://static/resource/2").await?;

    Ok(())
}
```
## Type Conversion

Commune provides convenient type conversion implementations for various inference APIs, including:

- AWS Bedrock
- OpenAI (coming soon)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the [MIT License](LICENSE).
