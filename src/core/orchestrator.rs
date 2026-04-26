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

pub async fn compact_history(mut history: Vec<crate::core::types::ChatMessage>) -> Vec<crate::core::types::ChatMessage> {
    if history.len() <= 10 {
        return history;
    }

    let mut to_compact = Vec::new();
    let mut indices_to_remove = Vec::new();

    for (i, msg) in history.iter().enumerate() {
        if matches!(msg.role, Role::User | Role::Assistant) {
            to_compact.push(msg.content.clone());
            indices_to_remove.push(i);
            if to_compact.len() == 5 {
                break;
            }
        }
    }

    if to_compact.is_empty() {
        return history;
    }

    let mut compacted_text = String::from("[SYSTEM: The following context was compacted: ");
    for text in to_compact {
        let snippet = if text.len() > 50 {
            &text[..50]
        } else {
            &text
        };
        compacted_text.push_str(snippet);
        compacted_text.push_str(" | ");
    }
    compacted_text.push_str("]");

    // Remove in reverse order to keep indices valid
    for &i in indices_to_remove.iter().rev() {
        history.remove(i);
    }

    // Insert the compacted message where the first message was removed
    if let Some(&first_idx) = indices_to_remove.first() {
        history.insert(first_idx, ChatMessage {
            role: Role::System,
            content: compacted_text,
        });
    }

    history
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
