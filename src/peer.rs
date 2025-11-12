use crate::error::Error;
use mcp_sdk_rs::{
    client::Client as McpClient,
    session::Session,
    transport::{websocket::WebSocketTransport, Message},
    types::ServerCapabilities,
    Implementation, LoggingLevel, Prompt, PromptMessage, Resource, ResourceContents,
    ResourceTemplate, Tool, ToolResult,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, fmt, process::Stdio, sync::Arc};
use tokio::{
    process::Command,
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        Mutex,
    },
};

#[derive(Default, Deserialize)]
pub struct LocalPeerBuilder {
    name: String,
    description: String,
    cmd: String,
    args: Vec<String>,
    env: HashMap<String, String>,
}
impl LocalPeerBuilder {
    pub fn new() -> LocalPeerBuilder {
        LocalPeerBuilder::default()
    }
    pub fn with_name(mut self, name: String) -> LocalPeerBuilder {
        self.name = name;
        self
    }
    pub fn with_description(mut self, description: String) -> LocalPeerBuilder {
        self.description = description;
        self
    }
    pub fn with_cmd(mut self, cmd: String) -> LocalPeerBuilder {
        self.cmd = cmd;
        self
    }
    pub fn with_args(mut self, args: Vec<String>) -> LocalPeerBuilder {
        self.args = args;
        self
    }
    pub fn with_env(mut self, env: HashMap<String, String>) -> LocalPeerBuilder {
        self.env = env;
        self
    }
    pub async fn build(self) -> Result<Peer, Error> {
        let mut command = Command::new(self.cmd.clone());
        command.args(self.args.clone());
        command.envs(self.env.clone());
        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        let (request_tx, request_rx): (UnboundedSender<Message>, UnboundedReceiver<Message>) =
            tokio::sync::mpsc::unbounded_channel();
        let (response_tx, response_rx): (UnboundedSender<Message>, UnboundedReceiver<Message>) =
            tokio::sync::mpsc::unbounded_channel();
        let session = Session::Local {
            handler: None,
            command: command,
            receiver: Arc::new(Mutex::new(request_rx)),
            sender: Arc::new(response_tx),
        };
        session.start().await.map_err(|_| Error::Internal)?;
        let client = McpClient::new(request_tx, response_rx);
        let implementation = Implementation {
            name: "commune".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        let caps = client
            .initialize(implementation, None)
            .await
            .map_err(|_| Error::ClientInitialization)?;
        log::debug!(
            "connected to local peer '{}'; capabilities: {:?}",
            self.name,
            caps
        );
        Ok(Peer::Local {
            name: self.name,
            description: self.description,
            cmd: self.cmd,
            args: self.args,
            env: self.env,
            capabilities: caps,
            client: Some(client),
        })
    }
}

#[derive(Default, Deserialize)]
pub struct RemotePeerBuilder {
    name: String,
    url: String,
    description: String,
}
impl RemotePeerBuilder {
    pub fn new() -> RemotePeerBuilder {
        RemotePeerBuilder::default()
    }
    pub fn with_name(mut self, name: String) -> RemotePeerBuilder {
        self.name = name;
        self
    }
    pub fn with_url(mut self, url: String) -> RemotePeerBuilder {
        self.url = url;
        self
    }
    pub fn with_description(mut self, description: String) -> RemotePeerBuilder {
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
        let session = Session::Remote {
            handler: None,
            transport: Arc::new(transport),
            receiver: Arc::new(Mutex::new(request_rx)),
            sender: Arc::new(response_tx),
        };
        // let session = Session::new(Arc::new(transport), response_tx, request_rx, None);
        session
            .start()
            .await
            .map_err(|_| Error::ClientInitialization)?;
        let client = McpClient::new(request_tx, response_rx);
        let implementation = Implementation {
            name: "commune".to_string(),
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
        Ok(Peer::Remote {
            name: self.name,
            url: self.url,
            description: self.description,
            capabilities: caps,
            client: Some(client),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Peer {
    Local {
        name: String,
        description: String,
        cmd: String,
        args: Vec<String>,
        env: HashMap<String, String>,
        capabilities: ServerCapabilities,
        #[serde(skip)]
        client: Option<McpClient>,
    },
    Remote {
        name: String,
        description: String,
        url: String,
        capabilities: ServerCapabilities,
        #[serde(skip)]
        client: Option<McpClient>,
    },
}

impl Peer {
    /// List available tools
    pub async fn list_tools(&self) -> Result<Vec<Tool>, Error> {
        match self {
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
            Peer::Remote {
                name: _,
                description: _,
                url: _,
                capabilities,
                client: _,
            } => {
                if capabilities.tools.is_some() {
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
        }
    }
    /// Call a tool
    pub async fn call_tool(&self, name: &str, params: Option<Value>) -> Result<ToolResult, Error> {
        match self {
            Peer::Local {
                name: _,
                description: _,
                cmd: _,
                args: _,
                env: _,
                capabilities,
                client,
            } => {
                if capabilities.tools.is_some() {
                    if let Some(c) = &client {
                        let val = c
                            .request(
                                "tools/call",
                                Some(json!({
                                    "name": name,
                                    "arguments": params.unwrap_or(json!({}))
                                })),
                            )
                            .await
                            .map_err(|e| Error::McpClient(format!("{e}")))?;
                        let tr: ToolResult =
                            serde_json::from_value(val).expect("an mcp formatted tool result");
                        Ok(tr)
                    } else {
                        Err(Error::UninitializedClient)
                    }
                } else {
                    Err(Error::Unsupported)
                }
            }
            Peer::Remote {
                name: _,
                description: _,
                url: _,
                capabilities,
                client,
            } => {
                if capabilities.tools.is_some() {
                    if let Some(c) = &client {
                        let val = c
                            .request(
                                "tools/call",
                                Some(json!({
                                    "name": name,
                                    "arguments": params.unwrap_or(json!({}))
                                })),
                            )
                            .await
                            .map_err(|e| Error::McpClient(format!("{e}")))?;
                        let tr: ToolResult =
                            serde_json::from_value(val).expect("an mcp formatted tool result");
                        Ok(tr)
                    } else {
                        Err(Error::UninitializedClient)
                    }
                } else {
                    Err(Error::Unsupported)
                }
            }
        }
    }
    /// List available resources
    pub async fn list_resources(&self) -> Result<Vec<Resource>, Error> {
        match self {
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
            Peer::Remote {
                name: _,
                description: _,
                url: _,
                capabilities,
                client: _,
            } => {
                if capabilities.resources.is_some() {
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
        }
    }
    /// Retrieve resource contents
    pub async fn get_resource(&self, uri: &str) -> Result<Vec<ResourceContents>, Error> {
        match self {
            Peer::Local {
                name: _,
                description: _,
                cmd: _,
                args: _,
                env: _,
                capabilities,
                client,
            } => {
                if capabilities.resources.is_some() {
                    if let Some(c) = &client {
                        let value = c
                            .request("resources/read", Some(json!({"uri": uri})))
                            .await
                            .map_err(|_| Error::McpClient("failed to read resource".to_string()))?;
                        let resource_obj: HashMap<String, Value> =
                            serde_json::from_value(value).map_err(|_| Error::InvalidResponse)?;
                        if let Some(val) = resource_obj.get("contents") {
                            let contents: Vec<ResourceContents> =
                                serde_json::from_value(val.clone())
                                    .map_err(|_| Error::InvalidResponse)?;
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
            Peer::Remote {
                name: _,
                description: _,
                url: _,
                capabilities,
                client,
            } => {
                if capabilities.resources.is_some() {
                    if let Some(c) = &client {
                        let value = c
                            .request("resources/read", Some(json!({"uri": uri})))
                            .await
                            .map_err(|_| Error::McpClient("failed to read resource".to_string()))?;
                        let resource_obj: HashMap<String, Value> =
                            serde_json::from_value(value).map_err(|_| Error::InvalidResponse)?;
                        if let Some(val) = resource_obj.get("contents") {
                            let contents: Vec<ResourceContents> =
                                serde_json::from_value(val.clone())
                                    .map_err(|_| Error::InvalidResponse)?;
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
        }
    }
    /// List resource templates
    pub async fn list_resource_templates(&self) -> Result<Vec<ResourceTemplate>, Error> {
        match self {
            Peer::Local {
                name: _,
                description: _,
                cmd: _,
                args: _,
                env: _,
                capabilities,
                client,
            } => {
                if capabilities.resources.is_some() {
                    if let Some(c) = &client {
                        let value =
                            c.request("resources/templates/list", None)
                                .await
                                .map_err(|_| {
                                    Error::McpClient("failed to list templates".to_string())
                                })?;
                        let template_obj: HashMap<String, Value> =
                            serde_json::from_value(value).map_err(|_| Error::InvalidResponse)?;
                        if let Some(val) = template_obj.get("resourceTemplates") {
                            let contents: Vec<ResourceTemplate> =
                                serde_json::from_value(val.clone())
                                    .map_err(|_| Error::InvalidResponse)?;
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
            Peer::Remote {
                name: _,
                description: _,
                url: _,
                capabilities,
                client,
            } => {
                if capabilities.resources.is_some() {
                    if let Some(c) = &client {
                        let value =
                            c.request("resources/templates/list", None)
                                .await
                                .map_err(|_| {
                                    Error::McpClient("failed to list templates".to_string())
                                })?;
                        let template_obj: HashMap<String, Value> =
                            serde_json::from_value(value).map_err(|_| Error::InvalidResponse)?;
                        if let Some(val) = template_obj.get("resourceTemplates") {
                            let contents: Vec<ResourceTemplate> =
                                serde_json::from_value(val.clone())
                                    .map_err(|_| Error::InvalidResponse)?;
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
        }
    }
    /// Subscribe to resource update notifications
    pub async fn subscribe(&self, uri: &str) -> Result<(), Error> {
        match self {
            Peer::Local {
                name: _,
                description: _,
                cmd: _,
                args: _,
                env: _,
                capabilities: _,
                client,
            } => {
                if let Some(c) = &client {
                    c.subscribe(uri).await.map_err(|_| {
                        Error::McpClient("failed to subscribe to update notifications".to_string())
                    })?;
                }
                Ok(())
            }
            Peer::Remote {
                name: _,
                description: _,
                url: _,
                capabilities: _,
                client,
            } => {
                if let Some(c) = &client {
                    c.subscribe(uri).await.map_err(|_| {
                        Error::McpClient("failed to subscribe to update notifications".to_string())
                    })?;
                }
                Ok(())
            }
        }
    }
    /// List available prompts
    pub async fn list_prompts(&self) -> Result<Vec<Prompt>, Error> {
        match self {
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
            Peer::Remote {
                name: _,
                description: _,
                url: _,
                capabilities,
                client: _,
            } => {
                if capabilities.prompts.is_some() {
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
        }
    }
    /// Retrieve prompt contents
    pub async fn get_prompt(
        &self,
        name: &str,
        args: Option<Value>,
    ) -> Result<Vec<PromptMessage>, Error> {
        match self {
            Peer::Local {
                name: _,
                description: _,
                cmd: _,
                args: _,
                env: _,
                capabilities,
                client,
            } => {
                if capabilities.prompts.is_some() {
                    if let Some(c) = &client {
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
                            let prompt: Vec<PromptMessage> = serde_json::from_value(val.clone())
                                .map_err(|_| Error::InvalidResponse)?;
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
            Peer::Remote {
                name: _,
                description: _,
                url: _,
                capabilities,
                client,
            } => {
                if capabilities.prompts.is_some() {
                    if let Some(c) = &client {
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
                            let prompt: Vec<PromptMessage> = serde_json::from_value(val.clone())
                                .map_err(|_| Error::InvalidResponse)?;
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
        }
    }

    pub async fn set_log_level(&self, level: LoggingLevel) -> Result<(), Error> {
        match self {
            Peer::Local {
                name: _,
                description: _,
                cmd: _,
                args: _,
                env: _,
                capabilities,
                client,
            } => {
                if capabilities.logging.is_some() {
                    if let Some(c) = &client {
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
            Peer::Remote {
                name: _,
                description: _,
                url: _,
                capabilities,
                client,
            } => {
                if capabilities.logging.is_some() {
                    if let Some(c) = &client {
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
        }
    }

    /// Perform a paginated request
    async fn paginated_request(&self, thing: &str) -> Result<Vec<Value>, Error> {
        match self {
            Peer::Local {
                name: _,
                description: _,
                cmd: _,
                args: _,
                env: _,
                capabilities: _,
                client,
            } => {
                if let Some(client) = &client {
                    let mut res: Vec<Value> = vec![];
                    let mut next_cursor: Option<String> = None;
                    let path = format!("{}/list", thing);
                    let value = client.request(path.as_str(), None).await.map_err(|_| {
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
            Peer::Remote {
                name: _,
                description: _,
                url: _,
                capabilities: _,
                client,
            } => {
                if let Some(client) = &client {
                    let mut res: Vec<Value> = vec![];
                    let mut next_cursor: Option<String> = None;
                    let path = format!("{}/list", thing);
                    let value = client.request(path.as_str(), None).await.map_err(|_| {
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
    }
}

impl PartialEq for Peer {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Peer::Local {
                name: _,
                description: _,
                cmd,
                args: _,
                env: _,
                capabilities: _,
                client: _,
            } => match other {
                Peer::Local {
                    name: _,
                    description: _,
                    cmd: other_cmd,
                    args: _,
                    env: _,
                    capabilities: _,
                    client: _,
                } => cmd == other_cmd,
                Peer::Remote {
                    name: _,
                    description: _,
                    url: _,
                    capabilities: _,
                    client: _,
                } => false,
            },
            Peer::Remote {
                name: _,
                description: _,
                url,
                capabilities: _,
                client: _,
            } => match other {
                Peer::Local {
                    name: _,
                    description: _,
                    cmd: _,
                    args: _,
                    env: _,
                    capabilities: _,
                    client: _,
                } => false,
                Peer::Remote {
                    name: _,
                    description: _,
                    url: other_url,
                    capabilities: _,
                    client: _,
                } => url == other_url,
            },
        }
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
