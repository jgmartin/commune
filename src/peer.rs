use crate::error::Error;
use mcp_sdk_rs::{
    client::{Client as McpClient, Session},
    transport::{websocket::WebSocketTransport, Message},
    types::ServerCapabilities,
    Implementation, LoggingLevel, Prompt, PromptMessage, Resource, ResourceContents,
    ResourceTemplate, Tool,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, fmt, sync::Arc};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

#[derive(Default, Deserialize)]
pub struct PeerBuilder {
    name: String,
    url: String,
    description: String,
}
impl PeerBuilder {
    pub fn new() -> PeerBuilder {
        PeerBuilder::default()
    }
    pub fn with_name(mut self, name: String) -> PeerBuilder {
        self.name = name;
        self
    }
    pub fn with_url(mut self, url: String) -> PeerBuilder {
        self.url = url;
        self
    }
    pub fn with_description(mut self, description: String) -> PeerBuilder {
        self.description = description;
        self
    }
    pub async fn build(self) -> Result<Peer, Error> {
        let transport = WebSocketTransport::new(self.url.as_str())
            .await
            .map_err(|_| Error::Internal)?;
        let (request_tx, request_rx): (UnboundedSender<Message>, UnboundedReceiver<Message>) =
            tokio::sync::mpsc::unbounded_channel();
        let (response_tx, response_rx): (UnboundedSender<Message>, UnboundedReceiver<Message>) =
            tokio::sync::mpsc::unbounded_channel();
        let session = Session::new(Arc::new(transport), response_tx, request_rx, None);
        session.start().await.map_err(|_| Error::Internal)?;
        let client = McpClient::new(request_tx, response_rx);
        let implementation = Implementation {
            name: self.name.clone(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        let caps = client
            .initialize(implementation, None)
            .await
            .map_err(|_| Error::ClientInitialization)?;
        log::debug!(
            "connected to peer '{}' @ {}; capabilities: {:?}",
            self.name,
            self.url,
            caps
        );
        Ok(Peer {
            name: self.name,
            url: self.url,
            description: self.description,
            capabilities: caps,
            client: Some(client),
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Peer {
    pub name: String,
    pub url: String,
    pub description: String,
    pub capabilities: ServerCapabilities,
    #[serde(skip)]
    pub client: Option<McpClient>,
}
impl Peer {
    /// List available tools
    pub async fn list_tools(&self) -> Result<Vec<Tool>, Error> {
        if self.capabilities.tools.is_some() {
            let res = self
                .paginated_request("tools")
                .await
                .map_err(|_| Error::Internal)?;
            let tools: Vec<Tool> = res
                .into_iter()
                .map(|r| serde_json::from_value(r).unwrap())
                .collect();
            Ok(tools)
        } else {
            Err(Error::Unsupported)
        }
    }
    /// Call a tool
    pub async fn call_tool(&self, name: &str, params: Option<Value>) -> Result<Value, Error> {
        if self.capabilities.tools.is_some() {
            if let Some(c) = &self.client {
                c.request(
                    "tools/call",
                    Some(json!({
                        "name": name,
                        "arguments": params.unwrap_or(json!({}))
                    })),
                )
                .await
                .map_err(|e| Error::McpClient(format!("{e}")))
            } else {
                Err(Error::UninitializedClient)
            }
        } else {
            Err(Error::Unsupported)
        }
    }
    /// List available resources
    pub async fn list_resources(&self) -> Result<Vec<Resource>, Error> {
        if self.capabilities.resources.is_some() {
            let res = self.paginated_request("resources").await?;
            let resources: Vec<Resource> = res
                .into_iter()
                .map(|r| serde_json::from_value(r).unwrap())
                .collect();
            Ok(resources)
        } else {
            Err(Error::Unsupported)
        }
    }
    /// Retrieve resource contents
    pub async fn get_resource(&self, uri: &str) -> Result<Vec<ResourceContents>, Error> {
        if self.capabilities.resources.is_some() {
            if let Some(c) = &self.client {
                let value = c
                    .request("resources/read", Some(json!({"uri": uri})))
                    .await
                    .map_err(|_| Error::McpClient("failed to read resource".to_string()))?;
                let resource_obj: HashMap<String, Value> =
                    serde_json::from_value(value).map_err(|_| Error::InvalidResponse)?;
                if let Some(val) = resource_obj.get("contents") {
                    let contents: Vec<ResourceContents> =
                        serde_json::from_value(val.clone()).map_err(|_| Error::InvalidResponse)?;
                    Ok(contents)
                } else {
                    Ok(vec![])
                }
            } else {
                Err(Error::UninitializedClient)
            }
        } else {
            Err(Error::Unsupported)
        }
    }
    /// List resource templates
    pub async fn list_resource_templates(&self) -> Result<Vec<ResourceTemplate>, Error> {
        if self.capabilities.resources.is_some() {
            if let Some(c) = &self.client {
                let value = c
                    .request("resources/templates/list", None)
                    .await
                    .map_err(|_| Error::McpClient("failed to list templates".to_string()))?;
                let template_obj: HashMap<String, Value> =
                    serde_json::from_value(value).map_err(|_| Error::InvalidResponse)?;
                if let Some(val) = template_obj.get("resourceTemplates") {
                    let contents: Vec<ResourceTemplate> =
                        serde_json::from_value(val.clone()).map_err(|_| Error::InvalidResponse)?;
                    Ok(contents)
                } else {
                    Ok(vec![])
                }
            } else {
                Err(Error::UninitializedClient)
            }
        } else {
            Err(Error::Unsupported)
        }
    }
    /// Subscribe to resource update notifications
    pub async fn subscribe(&self, uri: &str) -> Result<(), Error> {
        if let Some(c) = &self.client {
            c.subscribe(uri).await.map_err(|_| {
                Error::McpClient("failed to subscribe to update notifications".to_string())
            })?;
        }
        Ok(())
    }
    /// List available prompts
    pub async fn list_prompts(&self) -> Result<Vec<Prompt>, Error> {
        if self.capabilities.prompts.is_some() {
            let res = self.paginated_request("prompts").await?;
            let prompts: Vec<Prompt> = res
                .into_iter()
                .map(|r| serde_json::from_value(r).unwrap())
                .collect();
            Ok(prompts)
        } else {
            Err(Error::Unsupported)
        }
    }
    /// Retrieve prompt contents
    pub async fn get_prompt(
        &self,
        name: &str,
        args: Option<Value>,
    ) -> Result<Vec<PromptMessage>, Error> {
        if self.capabilities.prompts.is_some() {
            if let Some(c) = &self.client {
                let value = c
                    .request(
                        "prompts/get",
                        Some(json!({"name": name, "arguments": args})),
                    )
                    .await
                    .map_err(|_| Error::McpClient("failed to get prompt".to_string()))?;
                let prompt_obj: HashMap<String, Value> =
                    serde_json::from_value(value).map_err(|_| Error::InvalidResponse)?;
                if let Some(val) = prompt_obj.get("messages") {
                    let prompt: Vec<PromptMessage> =
                        serde_json::from_value(val.clone()).map_err(|_| Error::InvalidResponse)?;
                    Ok(prompt)
                } else {
                    Err(Error::InvalidResponse)
                }
            } else {
                Err(Error::UninitializedClient)
            }
        } else {
            Err(Error::Unsupported)
        }
    }

    pub async fn set_log_level(&self, level: LoggingLevel) -> Result<(), Error> {
        if self.capabilities.logging.is_some() {
            if let Some(c) = &self.client {
                c.set_log_level(level)
                    .await
                    .map_err(|_| Error::McpClient("failed to set log level".to_string()))
            } else {
                Err(Error::UninitializedClient)
            }
        } else {
            Err(Error::Unsupported)
        }
    }

    /// Perform a paginated request
    async fn paginated_request(&self, thing: &str) -> Result<Vec<Value>, Error> {
        if let Some(client) = &self.client {
            let mut res: Vec<Value> = vec![];
            let mut next_cursor: Option<String> = None;
            let path = format!("{}/list", thing);
            let value = client
                .request(path.as_str(), None)
                .await
                .map_err(|_| Error::McpClient("failed to perform paginated request".to_string()))?;
            let resp_obj: HashMap<String, Value> =
                serde_json::from_value(value).map_err(|_| Error::InvalidResponse)?;
            if let Some(val) = resp_obj.get(thing) {
                if let Some(arr) = val.clone().as_array_mut() {
                    res.append(arr);
                }
                if let Some(nc_val) = resp_obj.get("nextCursor") {
                    if let Some(nc) = nc_val.as_str() {
                        next_cursor = Some(nc.to_string());
                    }
                }
            }
            while let Some(ref c) = next_cursor {
                let value = client
                    .request(path.as_str(), Some(json!({ "cursor": c })))
                    .await
                    .map_err(|_| {
                        Error::McpClient("failed to perform paginated request".to_string())
                    })?;
                let resp_obj: HashMap<String, Value> =
                    serde_json::from_value(value).map_err(|_| Error::InvalidResponse)?;
                if let Some(val) = resp_obj.get(thing) {
                    if let Some(arr) = val.clone().as_array_mut() {
                        res.append(arr);
                    }
                    if let Some(nc_val) = resp_obj.get("nextCursor") {
                        if let Some(nc) = nc_val.as_str() {
                            next_cursor = Some(nc.to_string());
                        } else {
                            next_cursor = None;
                        }
                    } else {
                        next_cursor = None;
                    }
                }
            }
            Ok(res)
        } else {
            Err(Error::UninitializedClient)
        }
    }
}

impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

pub struct PeerResource {
    pub peer: Peer,
    pub resource: Resource,
}

impl fmt::Display for PeerResource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.resource.name)
    }
}

pub struct PeerPrompt {
    pub peer: Peer,
    pub prompt: Prompt,
}

impl fmt::Display for PeerPrompt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.prompt.name)
    }
}
