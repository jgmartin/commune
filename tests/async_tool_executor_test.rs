//! Tests for the new async tool executor functionality

use mcp_commune::{
    error::Error,
    tool::{async_fn_executor, sync_fn_executor, Tool, Executor},
};
use mcp_sdk_rs::types::{MessageContent, Tool as McpTool};
use serde_json::{json, Value};

#[tokio::test]
async fn test_sync_fn_executor() {
    // Define a synchronous function
    fn greet_sync(params: &Option<Value>) -> Result<MessageContent, Error> {
        let name = params
            .as_ref()
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("World");
            
        Ok(MessageContent::Text {
            text: format!("Hello, {}!", name),
        })
    }
    
    // Create a tool with sync executor
    let executor = sync_fn_executor(greet_sync);
    let mcp_tool = McpTool {
        name: "greet_sync".to_string(),
        description: "A synchronous greeting tool".to_string(),
        input_schema: None,
        annotations: None,
    };
    
    let tool = Tool::Local {
        executor,
        tool: mcp_tool,
    };
    
    // Test the tool execution through switchboard's call_tool pattern
    if let Tool::Local { executor, tool: _ } = &tool {
        if let Executor::Fn(exec_fn) = executor {
            let params = Some(json!({"name": "Alice"}));
            let result = exec_fn(params).await;
            
            match result {
                Ok(MessageContent::Text { text }) => {
                    assert_eq!(text, "Hello, Alice!");
                }
                _ => panic!("Expected text result"),
            }
        }
    }
}

#[tokio::test]
async fn test_async_fn_executor() {
    // Define an async function
    async fn greet_async(params: Option<Value>) -> Result<MessageContent, Error> {
        let name = params
            .as_ref()
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("World");
            
        // Simulate async work
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            
        Ok(MessageContent::Text {
            text: format!("Hello async, {}!", name),
        })
    }
    
    // Create a tool with async executor
    let executor = async_fn_executor(greet_async);
    let mcp_tool = McpTool {
        name: "greet_async".to_string(),
        description: "An asynchronous greeting tool".to_string(),
        input_schema: None,
        annotations: None,
    };
    
    let tool = Tool::Local {
        executor,
        tool: mcp_tool,
    };
    
    // Test the tool execution
    if let Tool::Local { executor, tool: _ } = &tool {
        if let Executor::Fn(exec_fn) = executor {
            let params = Some(json!({"name": "Bob"}));
            let result = exec_fn(params).await;
            
            match result {
                Ok(MessageContent::Text { text }) => {
                    assert_eq!(text, "Hello async, Bob!");
                }
                _ => panic!("Expected text result"),
            }
        }
    }
}

#[tokio::test]
async fn test_closure_executor() {
    let context = "test-context".to_string();
    
    // Create an async executor from a closure that captures context
    let executor = async_fn_executor(move |params: Option<Value>| {
        let context = context.clone();
        async move {
            let message = params
                .as_ref()
                .and_then(|p| p.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Hello");
                
            Ok(MessageContent::Text {
                text: format!("{} - Context: {}", message, context),
            })
        }
    });
    
    let mcp_tool = McpTool {
        name: "context_tool".to_string(),
        description: "A tool that uses captured context".to_string(),
        input_schema: None,
        annotations: None,
    };
    
    let tool = Tool::Local {
        executor,
        tool: mcp_tool,
    };
    
    // Test the tool execution
    if let Tool::Local { executor, tool: _ } = &tool {
        if let Executor::Fn(exec_fn) = executor {
            let params = Some(json!({"message": "Hi there"}));
            let result = exec_fn(params).await;
            
            match result {
                Ok(MessageContent::Text { text }) => {
                    assert_eq!(text, "Hi there - Context: test-context");
                }
                _ => panic!("Expected text result"),
            }
        }
    }
}