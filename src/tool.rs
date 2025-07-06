use crate::{error::Error, peer::Peer};
use aws_sdk_bedrockruntime::types::{ToolInputSchema, ToolSpecification};
use aws_smithy_types::Document;
pub use mcp_sdk_rs::{MessageContent, Tool as McpTool, ToolResult};
use serde_json::Value;
use std::{collections::HashMap, fmt};

#[derive(Clone, Debug)]
pub enum Executor {
    Fn(fn(Option<Value>) -> Result<MessageContent, Error>),
    Cmd {
        cmd: String,
        args: Option<Vec<String>>,
    },
}

#[derive(Clone, Debug)]
pub enum Tool {
    Local { executor: Executor, tool: McpTool },
    Remote { peer: Peer, tool: McpTool },
}
impl fmt::Display for Tool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Tool::Remote { peer: _, tool } => write!(f, "{}", tool.description),
            Tool::Local { executor: _, tool } => write!(f, "{}", tool.description),
        }
    }
}
impl From<Tool> for ToolSpecification {
    fn from(tool: Tool) -> ToolSpecification {
        let mut properties = HashMap::new();
        let mut required = vec![];
        let name: String;
        let description: String;
        match tool {
            Tool::Remote { peer: _, tool } => {
                name = tool.name.clone();
                description = tool.description.clone();

                if let Some(is) = tool.input_schema {
                    if let Some(p) = is.properties {
                        if let Some(args_obj) = p.as_object() {
                            for (arg_name, val) in args_obj {
                                if let Some(schema_obj) = val.as_object() {
                                    properties
                                        .insert(arg_name.clone(), Document::Object(HashMap::new()));
                                    for (_, val) in schema_obj {
                                        if let Some(props_obj) = val.as_object() {
                                            if let Some(t) = props_obj.get("type") {
                                                if let Some(st) = t.as_str() {
                                                    if let Some(arg_props) =
                                                        properties.get_mut(arg_name)
                                                    {
                                                        if let Some(p) = arg_props.as_object_mut() {
                                                            p.insert(
                                                                "type".to_string(),
                                                                Document::String(st.to_string()),
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(r) = is.required {
                        for val in r {
                            required.push(Document::String(val))
                        }
                    }
                }
            }
            Tool::Local { executor: _, tool } => {
                name = tool.name.clone();
                description = tool.description.clone();
                if let Some(is) = tool.input_schema {
                    if let Some(p) = is.properties {
                        if let Some(args_obj) = p.as_object() {
                            for (arg_name, val) in args_obj {
                                if let Some(schema_obj) = val.as_object() {
                                    properties
                                        .insert(arg_name.clone(), Document::Object(HashMap::new()));
                                    for (_, val) in schema_obj {
                                        if let Some(props_obj) = val.as_object() {
                                            if let Some(t) = props_obj.get("type") {
                                                if let Some(st) = t.as_str() {
                                                    if let Some(arg_props) =
                                                        properties.get_mut(arg_name)
                                                    {
                                                        if let Some(p) = arg_props.as_object_mut() {
                                                            p.insert(
                                                                "type".to_string(),
                                                                Document::String(st.to_string()),
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(r) = is.required {
                        for val in r {
                            required.push(Document::String(val))
                        }
                    }
                }
            }
        }
        let input_schema_doc = Document::Object(HashMap::<String, Document>::from([
            ("type".into(), Document::String("object".into())),
            ("properties".into(), Document::Object(properties)),
            ("required".into(), Document::Array(required)),
        ]));
        ToolSpecification::builder()
            .set_name(Some(name))
            .set_description(Some(description))
            .set_input_schema(Some(ToolInputSchema::Json(input_schema_doc)))
            .build()
            .expect("a valid tool specification")
    }
}
