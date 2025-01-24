use crate::{
    error::Error,
    peer::{Peer, PeerBuilder, PeerPrompt, PeerResource, PeerTool},
};
use mcp_sdk_rs::types::ClientCapabilities;
use serde_json::json;
use std::collections::HashMap;

#[derive(Default)]
pub struct ClientBuilder {
    peers: Vec<Peer>,
    capabilities: ClientCapabilities,
}
impl ClientBuilder {
    pub fn new() -> ClientBuilder {
        // caps['experimental']['peers'] = Some(json!({})), Some(json!({"listChanged": true})) or None
        let caps = ClientCapabilities {
            experimental: Some(json!({"peers": {}})),
            ..Default::default()
        };
        ClientBuilder {
            capabilities: caps,
            ..Default::default()
        }
    }
    pub fn with_peers(mut self, peers: Vec<Peer>) -> ClientBuilder {
        self.peers = peers;
        self
    }
    pub fn with_peer(mut self, peer: Peer) -> ClientBuilder {
        self.peers.push(peer);
        self
    }
    pub fn with_capabilities(mut self, capabilities: ClientCapabilities) -> ClientBuilder {
        self.capabilities = capabilities;
        self
    }
    pub async fn build(self) -> Result<Client, Error> {
        if self.peers.is_empty() {
            log::error!("error: peer list cannot be empty");
            return Err(Error::Internal);
        }
        Ok(Client {
            peers: self.get_peers().await?,
        })
    }
    async fn get_peers(&self) -> Result<HashMap<String, Peer>, Error> {
        let mut new_peers = HashMap::new();
        for peer in &self.peers {
            if let Some(ref client) = peer.client {
                if let Some(ref caps) = peer.capabilities.experimental {
                    if let Some(x) = caps.get("experimental") {
                        if let Some(m) = x.as_object() {
                            if let Some(c) = m.get("commune") {
                                if let Some(b) = c.as_bool() {
                                    if b {
                                        log::debug!(
                                            "{} is a commune server, getting peers",
                                            peer.url
                                        );
                                        let r = client
                                            .request("peers/list", None)
                                            .await
                                            .map_err(|_| Error::McpClient)?;
                                        let remote_peers: Vec<PeerBuilder> =
                                            serde_json::from_value(r)
                                                .map_err(|_| Error::InvalidResponse)?;
                                        for peer_builder in remote_peers {
                                            let p = peer_builder.build().await?;
                                            new_peers.insert(p.name.clone(), p);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            log::debug!("{} is not a commune server", peer.url);
            new_peers.insert(peer.name.clone(), peer.clone());
        }
        Ok(new_peers)
    }
}

pub struct Client {
    pub peers: HashMap<String, Peer>,
}
impl Client {
    /// List all peer tools
    pub async fn all_tools(&self) -> Result<Vec<PeerTool>, Error> {
        let mut res = vec![];
        for peer in self.peers.values() {
            if peer.capabilities.tools.is_some() {
                for tool in peer.list_tools().await? {
                    res.push(PeerTool { peer, tool })
                }
            }
        }
        Ok(res)
    }

    /// List all peer resources
    pub async fn all_resources(&self) -> Result<Vec<PeerResource>, Error> {
        let mut res = vec![];
        for peer in self.peers.values() {
            if peer.capabilities.resources.is_some() {
                for resource in peer.list_resources().await? {
                    res.push(PeerResource { peer, resource })
                }
            }
        }
        Ok(res)
    }

    /// List all peer prompts
    pub async fn all_prompts(&self) -> Result<Vec<PeerPrompt>, Error> {
        let mut res = vec![];
        for peer in self.peers.values() {
            if peer.capabilities.prompts.is_some() {
                for prompt in peer.list_prompts().await? {
                    res.push(PeerPrompt { peer, prompt })
                }
            }
        }
        Ok(res)
    }
}
