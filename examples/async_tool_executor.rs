//! Example demonstrating how to use the new async tool executor
//!
//! This example shows how to create tools with both synchronous and asynchronous
//! executors using the new async function interface.

use mcp_commune::{
    error::Error,
    tool::{async_fn_executor, sync_fn_executor, Tool},
};
use mcp_sdk_rs::types::{MessageContent, Tool as McpTool};
use serde_json::Value;

fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        // Example 1: Creating a tool with a synchronous function
        let sync_tool = create_sync_tool();
        
        // Example 2: Creating a tool with an async function
        let async_tool = create_async_tool();
        
        println!("Created sync tool: {:?}", sync_tool);
        println!("Created async tool: {:?}", async_tool);
        
        // Both tools now have the same Executor::Fn interface internally
        // but can be called uniformly in an async context
        
        println!("Examples created successfully!");
    });
}

/// Example of creating a tool with a synchronous function
fn create_sync_tool() -> Tool {
    // Define a synchronous function that follows the old interface
    fn my_sync_function(params: &Option<Value>) -> Result<MessageContent, Error> {
        let name = params
            .as_ref()
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("World");
            
        Ok(MessageContent::Text {
            text: format!("Hello, {}! (from sync function)", name),
        })
    }
    
    // Convert the sync function to an async executor
    let executor = sync_fn_executor(my_sync_function);
    
    let mcp_tool = McpTool {
        name: "greet_sync".to_string(),
        description: "A synchronous greeting tool".to_string(),
        input_schema: None,
        annotations: None,
    };
    
    Tool::Local {
        executor,
        tool: mcp_tool,
    }
}

/// Example of creating a tool with an async function
fn create_async_tool() -> Tool {
    // Define an async function that can do real async work
    async fn my_async_function(params: Option<Value>) -> Result<MessageContent, Error> {
        let name = params
            .as_ref()
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("World");
            
        // Simulate some async work
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        Ok(MessageContent::Text {
            text: format!("Hello, {}! (from async function after 100ms)", name),
        })
    }
    
    // Create an async executor
    let executor = async_fn_executor(my_async_function);
    
    let mcp_tool = McpTool {
        name: "greet_async".to_string(),
        description: "An asynchronous greeting tool".to_string(),
        input_schema: None,
        annotations: None,
    };
    
    Tool::Local {
        executor,
        tool: mcp_tool,
    }
}

/// Example of creating a tool with a closure (useful for capturing context)
#[allow(dead_code)]
fn create_closure_tool(context: String) -> Tool {
    // Create an async executor from a closure that captures context
    let executor = async_fn_executor(move |params: Option<Value>| {
        let context = context.clone(); // Clone for move into async block
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
    
    Tool::Local {
        executor,
        tool: mcp_tool,
    }
}