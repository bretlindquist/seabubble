use crate::core::types::*;
use crate::mcp::client::McpClientImpl;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

pub async fn handle_gh_issues(repo: &str, tx: tokio::sync::mpsc::UnboundedSender<AppEvent>) {
    sleep(Duration::from_secs(1)).await;
    let mock_data = format!("Mock issue data for {}", repo);
    let result = ToolResult {
        id: "gh_issues_1".to_string(),
        output: mock_data,
        is_error: false,
    };
    if let Err(e) = tx.send(AppEvent::ToolCallResult(result)) {
        eprintln!("Failed to send tool call result: {}", e);
    }
}

pub async fn execute_tool(
    call: ToolCall,
    mcp_client: Arc<McpClientImpl>,
    tx: tokio::sync::mpsc::UnboundedSender<AppEvent>,
) {
    let call_id = call.id.clone();

    let result = match mcp_client
        .call_tool(&call.name, call.arguments.clone())
        .await
    {
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
