use crate::{
    error::Error,
    peer::{Peer, PeerBuilder, PeerPrompt, PeerResource, PeerTool},
};
use mcp_sdk_rs::types::ClientCapabilities;

#[derive(Default)]
pub struct ClientBuilder {
    peers: Vec<Peer>,
    capabilities: ClientCapabilities,
}
impl ClientBuilder {
    pub fn new() -> ClientBuilder {
        ClientBuilder::default()
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
    async fn get_peers(&self) -> Result<Vec<Peer>, Error> {
        let mut new_peers = self.peers.clone();
        for peer in &self.peers {
            if let Some(ref client) = peer.client {
                if let Some(ref caps) = peer.capabilities.experimental {
                    if let Some(x) = caps.as_object() {
                        if x.get("peers").is_some() {
                            log::debug!("{} is a commune server, getting peers", peer.url);
                            let r = client.request("peers/list", None).await.map_err(|_| {
                                Error::McpClient("failed to list peers".to_string())
                            })?;
                            let remote_peers: Vec<PeerBuilder> =
                                serde_json::from_value(r).map_err(|_| Error::InvalidResponse)?;
                            for pb in remote_peers {
                                new_peers.push(pb.build().await?);
                            }
                        }
                    }
                }
            }
        }
        Ok(new_peers)
    }
}

pub struct Client {
    pub peers: Vec<Peer>,
}
impl Client {
    /// List all peer tools
    pub async fn all_tools(&self) -> Result<Vec<PeerTool>, Error> {
        let mut res = vec![];
        for peer in &self.peers {
            if peer.capabilities.tools.is_some() {
                for tool in peer.list_tools().await? {
                    res.push(PeerTool {
                        peer: peer.clone(),
                        tool,
                    })
                }
            }
        }
        Ok(res)
    }

    /// List all peer resources
    pub async fn all_resources(&self) -> Result<Vec<PeerResource>, Error> {
        let mut res = vec![];
        for peer in &self.peers {
            if peer.capabilities.resources.is_some() {
                for resource in peer.list_resources().await? {
                    res.push(PeerResource {
                        peer: peer.clone(),
                        resource,
                    })
                }
            }
        }
        Ok(res)
    }

    /// List all peer prompts
    pub async fn all_prompts(&self) -> Result<Vec<PeerPrompt>, Error> {
        let mut res = vec![];
        for peer in &self.peers {
            if peer.capabilities.prompts.is_some() {
                for prompt in peer.list_prompts().await? {
                    res.push(PeerPrompt {
                        peer: peer.clone(),
                        prompt,
                    })
                }
            }
        }
        Ok(res)
    }
}
