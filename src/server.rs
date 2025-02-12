//! Server module for the Commune project.
//!
//! This module provides the implementation of the Commune server, including
//! the server builder, the main server struct, and the handler for WebSocket connections.

use crate::{error::Error, peer::Peer};
use async_trait::async_trait;
use mcp_sdk_rs::{
    error::Error as McpError,
    server::{Server as McpServer, ServerHandler},
    transport::websocket::WebSocketTransport,
    types::*,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

/// Builder for creating a Commune server with customizable components.
#[derive(Default)]
pub struct ServerBuilder {
    peers: Vec<Peer>,
    prompts: Vec<Prompt>,
    resources: Vec<Resource>,
    tools: Vec<Tool>,
}

impl ServerBuilder {
    /// Creates a new ServerBuilder with default values.
    pub fn new() -> ServerBuilder {
        ServerBuilder {
            ..Default::default()
        }
    }

    /// Adds multiple peers to the server.
    pub fn with_peers(mut self, peers: Vec<Peer>) -> ServerBuilder {
        self.peers = peers;
        self
    }

    /// Adds a single peer to the server.
    pub fn with_peer(mut self, peer: Peer) -> ServerBuilder {
        self.peers.push(peer);
        self
    }

    /// Adds a single prompt to the server.
    pub fn with_prompt(mut self, prompt: Prompt) -> ServerBuilder {
        self.prompts.push(prompt);
        self
    }

    /// Adds multiple prompts to the server.
    pub fn with_prompts(mut self, prompts: Vec<Prompt>) -> ServerBuilder {
        self.prompts = prompts;
        self
    }

    /// Adds a single resource to the server.
    pub fn with_resource(mut self, resource: Resource) -> ServerBuilder {
        self.resources.push(resource);
        self
    }

    /// Adds multiple resources to the server.
    pub fn with_resources(mut self, resources: Vec<Resource>) -> ServerBuilder {
        self.resources = resources;
        self
    }

    /// Adds a single tool to the server.
    pub fn with_tool(mut self, tool: Tool) -> ServerBuilder {
        self.tools.push(tool);
        self
    }

    /// Adds multiple tools to the server.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> ServerBuilder {
        self.tools = tools;
        self
    }

    /// Builds the server with the configured components.
    ///
    /// # Returns
    /// A `Result` containing either the built `Server` or an `Error`.
    pub async fn build(self) -> Result<Server, Error> {
        let mut caps = ServerCapabilities::default();
        // Set up server capabilities based on the presence of different components
        if !self.peers.is_empty() {
            caps.experimental = Some(json!({"peers": {}}));
        }
        if !self.prompts.is_empty() {
            caps.prompts = Some(json!({}));
        }
        if !self.resources.is_empty() {
            caps.resources = Some(json!({}));
        }
        if !self.tools.is_empty() {
            caps.tools = Some(json!({}));
        }
        Ok(Server {
            peers: self.peers,
            capabilities: caps,
            prompts: self.prompts,
            resources: self.resources,
            tools: self.tools,
        })
    }
}

/// The main Server struct representing a Commune server instance.
pub struct Server {
    pub capabilities: ServerCapabilities,
    pub peers: Vec<Peer>,
    pub prompts: Vec<Prompt>,
    pub resources: Vec<Resource>,
    pub tools: Vec<Tool>,
}

impl Server {
    /// Starts the server and listens for incoming WebSocket connections.
    ///
    /// # Arguments
    /// * `addr` - A string slice that holds the address to bind the server to.
    ///
    /// # Returns
    /// A `Result` indicating success or containing an `Error`.
    pub async fn start(&self, addr: &str) -> Result<(), Error> {
        let listener = TcpListener::bind(addr).await.map_err(|_| Error::Internal)?;
        println!("WebSocket server listening on ws://{}", addr);

        while let Ok((stream, addr)) = listener.accept().await {
            println!("New connection from: {}", addr);
            let ws_stream = accept_async(stream)
                .await
                .map_err(|e| {
                    println!("Error during WebSocket handshake: {}", e);
                    e
                })
                .unwrap();

            let transport = WebSocketTransport::from_stream(ws_stream);
            let handler = CommuneHandler {
                peers: self.peers.clone(),
                prompts: self.prompts.clone(),
                tools: self.tools.clone(),
                resources: self.resources.clone(),
            };
            let server = McpServer::new(Arc::new(transport), Arc::new(handler));

            tokio::spawn(async move {
                println!("Starting server for connection from {}", addr);
                if let Err(e) = server.start().await {
                    eprintln!("Error in WebSocket connection from {}: {}", addr, e);
                }
                println!("Connection from {} closed", addr);
            });
        }
        Ok(())
    }
}

/// Handler for Commune server requests.
struct CommuneHandler {
    peers: Vec<Peer>,
    prompts: Vec<Prompt>,
    tools: Vec<Tool>,
    resources: Vec<Resource>,
}

#[async_trait]
impl ServerHandler for CommuneHandler {
    /// Initializes the connection with a client.
    ///
    /// # Arguments
    /// * `implementation` - The client implementation details.
    /// * `_capabilities` - The client's capabilities (currently unused).
    ///
    /// # Returns
    /// A `Result` containing either the `ServerCapabilities` or an `McpError`.
    async fn initialize(
        &self,
        implementation: Implementation,
        _capabilities: ClientCapabilities,
    ) -> Result<ServerCapabilities, McpError> {
        println!(
            "Client connected: {} v{}",
            implementation.name, implementation.version
        );
        Ok(ServerCapabilities::default())
    }

    /// Handles incoming method calls from clients.
    ///
    /// # Arguments
    /// * `method` - A string slice containing the method name.
    /// * `_params` - An optional `Value` containing the method parameters.
    ///
    /// # Returns
    /// A `Result` containing either the method result as a `Value` or an `McpError`.
    async fn handle_method(&self, method: &str, _params: Option<Value>) -> Result<Value, McpError> {
        match method {
            "peers/list" => Ok(serde_json::to_value(ListPeersResult {
                peers: self.peers.clone(),
                next_cursor: None,
            })?),
            "prompts/list" => Ok(serde_json::to_value(ListPromptsResult {
                prompts: self.prompts.clone(),
                next_cursor: None,
            })?),
            "tools/list" => Ok(serde_json::to_value(ListToolsResult {
                tools: self.tools.clone(),
                next_cursor: None,
            })?),
            "resources/list" => Ok(serde_json::to_value(ListResourcesResult {
                resources: self.resources.clone(),
                next_cursor: None,
            })?),
            _ => Err(McpError::Other("unknown method".to_string())),
        }
    }

    /// Handles the shutdown request from a client.
    ///
    /// # Returns
    /// A `Result` indicating success or containing an `McpError`.
    async fn shutdown(&self) -> Result<(), McpError> {
        println!("Server shutting down");
        Ok(())
    }
}

/// Request structure for the /peers/list method.
#[derive(Clone, Serialize, Deserialize)]
pub struct ListPeersRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
}

/// Response structure for the /peers/list method.
#[derive(Clone, Serialize, Deserialize)]
pub struct ListPeersResult {
    pub peers: Vec<Peer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<Cursor>,
}