use crate::{error::Error, peer::Peer};
use aws_sdk_bedrockruntime::types::{ToolInputSchema, ToolSpecification};
use aws_smithy_types::Document;
pub use mcp_sdk_rs::{MessageContent, Tool as McpTool, ToolResult};
use serde_json::Value;
use std::fmt;

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
        let name: String;
        let description: String;
        let mut input_schema: std::collections::HashMap<String, Document> =
            std::collections::HashMap::default();
        input_schema.insert(
            "properties".to_string(),
            Document::Object(std::collections::HashMap::default()),
        );
        input_schema.insert("required".to_string(), Document::Array(vec![]));
        match tool {
            Tool::Remote { peer: _, tool } => {
                name = tool.name.clone();
                description = tool.description.clone();
                if let Some(schema) = &tool.input_schema {
                    if let Some(props) = &schema.properties {
                        let props_val =
                            serde_json::to_value(props).expect("a serializable tool schema");
                        let props_doc: Document =
                            serde_json::from_value(props_val).expect("a valid tool schema");
                        input_schema.insert("properties".to_string(), props_doc);
                    }
                    if let Some(req) = &schema.required {
                        let required_val =
                            serde_json::to_value(req).expect("serializable required params");
                        let required_doc: Document = serde_json::from_value(required_val)
                            .expect("valid required parameters");
                        input_schema.insert("required".to_string(), required_doc);
                    }
                }
            }
            Tool::Local { executor: _, tool } => {
                name = tool.name.clone();
                description = tool.description.clone();
                if let Some(schema) = &tool.input_schema {
                    if let Some(props) = &schema.properties {
                        let props_val =
                            serde_json::to_value(props).expect("a serializable tool schema");
                        let props_doc: Document =
                            serde_json::from_value(props_val).expect("a valid tool schema");
                        input_schema.insert("properties".to_string(), props_doc);
                    }
                    if let Some(req) = &schema.required {
                        let required_val =
                            serde_json::to_value(req).expect("serializable required params");
                        let required_doc: Document = serde_json::from_value(required_val)
                            .expect("valid required parameters");
                        input_schema.insert("required".to_string(), required_doc);
                    }
                }
            }
        }
        input_schema.insert("type".to_string(), Document::String("object".to_string()));
        let input_schema_doc = Document::from(input_schema);
        ToolSpecification::builder()
            .set_name(Some(name))
            .set_description(Some(description))
            .set_input_schema(Some(ToolInputSchema::Json(input_schema_doc)))
            .build()
            .expect("a valid tool specification")
    }
}
