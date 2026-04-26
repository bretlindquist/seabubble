use std::sync::Arc;
use crate::mcp::client::McpClientImpl;
use crate::core::types::*;

pub async fn execute_tool(
    call: ToolCall,
    mcp_client: Arc<McpClientImpl>,
    tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
) {
    let call_id = call.id.clone();
    
    let result = match mcp_client.call_tool(&call.name, call.arguments.clone()).await {
        Ok(output) => ToolResult {
            id: call_id,
            output,
            is_error: false,
        },
        Err(e) => ToolResult {
            id: call_id,
            output: e.to_string(),
            is_error: true,
        },
    };

    if let Err(e) = tx.send(AppEvent::ToolCallResult(result)) {
        eprintln!("Failed to send tool call result: {}", e);
    }
}
