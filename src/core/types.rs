pub enum AppMode {
    Normal,
    Status,
    PermissionPrompt(ToolCall),
}

pub struct AppState {
    pub mode: AppMode,
}

#[derive(Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

pub struct ToolResult {
    pub id: String,
    pub output: String,
    pub is_error: bool,
}

pub enum AppEvent {
    ToolCallResult(ToolResult),
}

pub enum Role {
    User, System,
    Assistant,
}

pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}
