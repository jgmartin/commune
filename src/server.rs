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

#[derive(Default)]
pub struct ServerBuilder {
    peers: Vec<Peer>,
    prompts: Vec<Prompt>,
    resources: Vec<Resource>,
    tools: Vec<Tool>,
}
impl ServerBuilder {
    pub fn new() -> ServerBuilder {
        ServerBuilder {
            ..Default::default()
        }
    }
    pub fn with_peers(mut self, peers: Vec<Peer>) -> ServerBuilder {
        self.peers = peers;
        self
    }
    pub fn with_peer(mut self, peer: Peer) -> ServerBuilder {
        self.peers.push(peer);
        self
    }
    pub fn with_prompt(mut self, prompt: Prompt) -> ServerBuilder {
        self.prompts.push(prompt);
        self
    }
    pub fn with_prompts(mut self, prompts: Vec<Prompt>) -> ServerBuilder {
        self.prompts = prompts;
        self
    }
    pub fn with_resource(mut self, resource: Resource) -> ServerBuilder {
        self.resources.push(resource);
        self
    }
    pub fn with_resources(mut self, resources: Vec<Resource>) -> ServerBuilder {
        self.resources = resources;
        self
    }
    pub fn with_tool(mut self, tool: Tool) -> ServerBuilder {
        self.tools.push(tool);
        self
    }
    pub fn with_tools(mut self, tools: Vec<Tool>) -> ServerBuilder {
        self.tools = tools;
        self
    }
    pub async fn build(self) -> Result<Server, Error> {
        let mut caps = ServerCapabilities::default();
        // caps['experimental']['peers'] = Some({}), Some(json!({"listChanged": true})) or None
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

pub struct Server {
    pub capabilities: ServerCapabilities,
    pub peers: Vec<Peer>,
    pub prompts: Vec<Prompt>,
    pub resources: Vec<Resource>,
    pub tools: Vec<Tool>,
}
impl Server {
    /// Start a server and listen for requests
    pub async fn start(&self, addr: &str) -> Result<(), Error> {
        let listener = TcpListener::bind(addr).await.map_err(|_| Error::Internal)?;
        println!("WebSocket server listening on ws://{}", addr);
        // Accept and handle incoming connections
        while let Ok((stream, addr)) = listener.accept().await {
            println!("New connection from: {}", addr);
            // Create WebSocket stream from TCP connection
            let ws_stream = accept_async(stream)
                .await
                .map_err(|e| {
                    println!("Error during WebSocket handshake: {}", e);
                    e
                })
                .unwrap();

            // Create WebSocket transport from the accepted stream
            let transport = WebSocketTransport::from_stream(ws_stream);
            // Create MCP server with the transport and handler
            let handler = CommuneHandler {
                peers: self.peers.clone(),
                prompts: self.prompts.clone(),
                tools: self.tools.clone(),
                resources: self.resources.clone(),
            };
            let server = McpServer::new(Arc::new(transport), Arc::new(handler));

            // Handle this connection in a new task
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

struct CommuneHandler {
    peers: Vec<Peer>,
    prompts: Vec<Prompt>,
    tools: Vec<Tool>,
    resources: Vec<Resource>,
}
#[async_trait]
impl ServerHandler for CommuneHandler {
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
            // "prompts/get" => {
            //     if let Some(p) = params {
            //         let req: GetPromptRequest = serde_json::from_value(p)?;
            //         if let Some(p) = self.prompts.iter().find(|r| r.name == req.name) {
            //             Ok(serde_json::to_value(GetPromptResult {
            //                 description: p.description,
            //                 messages: p.messages,
            //             })?)
            //         } else {
            //             let e = McpError::Protocol {
            //                 code: ErrorCode::RequestFailed,
            //                 message: "prompt not found".to_string(),
            //                 data: None,
            //             };
            //             Err(e)
            //         }
            //     } else {
            //         let e = McpError::Protocol {
            //             code: ErrorCode::InvalidRequest,
            //             message: "invalid request".to_string(),
            //             data: None,
            //         };
            //         Err(e)
            //     }
            // }
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

    async fn shutdown(&self) -> Result<(), McpError> {
        println!("Server shutting down");
        Ok(())
    }
}

/// /peers/list request
#[derive(Clone, Serialize, Deserialize)]
pub struct ListPeersRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
}

/// /peers/list response
#[derive(Clone, Serialize, Deserialize)]
pub struct ListPeersResult {
    pub peers: Vec<Peer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<Cursor>,
}
