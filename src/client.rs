//! # Client Module
//!
//! This module provides the client-side functionality for the Commune library.
//! It includes structures and implementations for building and managing clients
//! that can interact with peers in a distributed network.

use crate::{
    error::Error,
    peer::{Peer, PeerPrompt, PeerResource, RemotePeerBuilder},
    tool::Tool,
};
use mcp_sdk_rs::types::ClientCapabilities;

/// A builder for creating `Client` instances with customizable configurations.
#[derive(Default)]
pub struct ClientBuilder {
    peers: Vec<Peer>,
    tools: Vec<Tool>,
    capabilities: ClientCapabilities,
}

impl ClientBuilder {
    /// Creates a new `ClientBuilder` instance.
    pub fn new() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Adds multiple peers to the client configuration.
    pub fn with_peers(mut self, peers: Vec<Peer>) -> ClientBuilder {
        self.peers.extend(peers);
        self
    }

    /// Adds a single peer to the client configuration.
    pub fn with_peer(mut self, peer: Peer) -> ClientBuilder {
        self.peers.push(peer);
        self
    }

    /// Adds multiple tools to the client configuration.
    /// Use this to add local tools.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> ClientBuilder {
        self.tools.extend(tools);
        self
    }

    /// Adds a single tool to the client configuration.
    /// Use this to add a local tool.
    pub fn with_tool(mut self, tool: Tool) -> ClientBuilder {
        self.tools.push(tool);
        self
    }

    /// Sets the capabilities for the client.
    pub fn with_capabilities(mut self, capabilities: ClientCapabilities) -> ClientBuilder {
        self.capabilities = capabilities;
        self
    }

    /// Builds the `Client` instance based on the configured parameters.
    ///
    /// # Errors
    /// Returns an error if the peer list is empty or if there's an issue retrieving peers.
    pub async fn build(self) -> Result<Client, Error> {
        Ok(Client {
            peers: self.get_peers().await?,
            local_tools: self.tools,
        })
    }

    /// Retrieves and aggregates peers, including those from Commune servers.
    ///
    /// # Errors
    /// Returns an error if there's an issue communicating with peers or parsing their responses.
    async fn get_peers(&self) -> Result<Vec<Peer>, Error> {
        let mut new_peers = self.peers.clone();
        for peer in &self.peers {
            match peer {
                Peer::Local {
                    name: _,
                    description: _,
                    cmd: _,
                    args: _,
                    env: _,
                    capabilities,
                    client,
                } => {
                    if let Some(ref client) = client {
                        if let Some(ref caps) = capabilities.experimental {
                            if let Some(x) = caps.as_object() {
                                if x.get("peers").is_some() {
                                    // log::debug!("{} is a commune server, getting peers", peer.url);
                                    let r =
                                        client.request("peers/list", None).await.map_err(|_| {
                                            Error::McpClient("failed to list peers".to_string())
                                        })?;
                                    let remote_peers: Vec<RemotePeerBuilder> =
                                        serde_json::from_value(r)
                                            .map_err(|_| Error::InvalidResponse)?;
                                    for pb in remote_peers {
                                        new_peers.push(pb.build().await?);
                                    }
                                }
                            }
                        }
                    }
                }
                Peer::Remote {
                    name: _,
                    description: _,
                    url: _,
                    capabilities,
                    client,
                } => {
                    if let Some(ref client) = client {
                        if let Some(ref caps) = capabilities.experimental {
                            if let Some(x) = caps.as_object() {
                                if x.get("peers").is_some() {
                                    // log::debug!("{} is a commune server, getting peers", peer.url);
                                    let r =
                                        client.request("peers/list", None).await.map_err(|_| {
                                            Error::McpClient("failed to list peers".to_string())
                                        })?;
                                    let remote_peers: Vec<RemotePeerBuilder> =
                                        serde_json::from_value(r)
                                            .map_err(|_| Error::InvalidResponse)?;
                                    for pb in remote_peers {
                                        new_peers.push(pb.build().await?);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(new_peers)
    }
}

/// Represents a client in the Commune network, capable of interacting with multiple peers.
pub struct Client {
    pub peers: Vec<Peer>,
    pub local_tools: Vec<Tool>,
}

impl Client {
    /// Lists all tools available across all connected peers.
    ///
    /// # Errors
    /// Returns an error if there's an issue communicating with any peer.
    pub async fn all_tools(&self) -> Result<Vec<Tool>, Error> {
        let mut res = self.local_tools.clone();
        for peer in &self.peers {
            match peer {
                Peer::Local {
                    name: _,
                    description: _,
                    cmd: _,
                    args: _,
                    env: _,
                    capabilities,
                    client: _,
                } => {
                    if capabilities.tools.is_some() {
                        for tool in peer.list_tools().await? {
                            // local peers speak MCP protocol and, as such, their tools are implemented as Tool::Remote
                            res.push(Tool::Remote {
                                peer: peer.clone(),
                                tool,
                            })
                        }
                    }
                }
                Peer::Remote {
                    name: _,
                    description: _,
                    url: _,
                    capabilities,
                    client: _,
                } => {
                    if capabilities.tools.is_some() {
                        for tool in peer.list_tools().await? {
                            res.push(Tool::Remote {
                                peer: peer.clone(),
                                tool,
                            })
                        }
                    }
                }
            }
        }
        Ok(res)
    }

    /// Lists all resources available across all connected peers.
    ///
    /// # Errors
    /// Returns an error if there's an issue communicating with any peer.
    pub async fn all_resources(&self) -> Result<Vec<PeerResource>, Error> {
        let mut res = vec![];
        for peer in &self.peers {
            match peer {
                Peer::Local {
                    name: _,
                    description: _,
                    cmd: _,
                    args: _,
                    env: _,
                    capabilities,
                    client: _,
                } => {
                    if capabilities.resources.is_some() {
                        for resource in peer.list_resources().await? {
                            res.push(PeerResource {
                                peer: peer.clone(),
                                resource,
                            })
                        }
                    }
                }
                Peer::Remote {
                    name: _,
                    description: _,
                    url: _,
                    capabilities,
                    client: _,
                } => {
                    if capabilities.resources.is_some() {
                        for resource in peer.list_resources().await? {
                            res.push(PeerResource {
                                peer: peer.clone(),
                                resource,
                            })
                        }
                    }
                }
            }
        }
        Ok(res)
    }

    /// Lists all prompts available across all connected peers.
    ///
    /// # Errors
    /// Returns an error if there's an issue communicating with any peer.
    pub async fn all_prompts(&self) -> Result<Vec<PeerPrompt>, Error> {
        let mut res = vec![];
        for peer in &self.peers {
            match peer {
                Peer::Local {
                    name: _,
                    description: _,
                    cmd: _,
                    args: _,
                    env: _,
                    capabilities,
                    client: _,
                } => {
                    if capabilities.prompts.is_some() {
                        for prompt in peer.list_prompts().await? {
                            res.push(PeerPrompt {
                                peer: peer.clone(),
                                prompt,
                            })
                        }
                    }
                }
                Peer::Remote {
                    name: _,
                    description: _,
                    url: _,
                    capabilities,
                    client: _,
                } => {
                    if capabilities.prompts.is_some() {
                        for prompt in peer.list_prompts().await? {
                            res.push(PeerPrompt {
                                peer: peer.clone(),
                                prompt,
                            })
                        }
                    }
                }
            }
        }
        Ok(res)
    }
}
