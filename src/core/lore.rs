use crate::core::types::{ChatMessage, Role};
use std::path::Path;
use tokio::fs;

pub async fn load_system_prompts(workspace: &Path) -> Vec<ChatMessage> {
    let files = [
        "AGENTS.md",
        "SOUL.md",
        "USER.md",
        "MEMORY.md",
        ".ct/BOOTSTRAP.md",
        "SEATURTLE.md",
    ];

    let mut messages = Vec::new();

    for filename in files {
        let file_path = workspace.join(filename);
        if let Ok(contents) = fs::read_to_string(&file_path).await {
            messages.push(ChatMessage {
                role: Role::System,
                content: format!("# File: {}\n\n{}", filename, contents),
            });
        }
    }

    messages
}
