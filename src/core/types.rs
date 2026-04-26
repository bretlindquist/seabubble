pub enum AppMode {
    Normal,
    Insert,
    Search,
    Status,
    Streaming,
    Steering,
    PermissionPrompt(ToolCall),
}

pub struct AppState {
    pub mode: AppMode,
    pub messages: Vec<ChatMessage>,
    pub input_buffer: String,
    pub search_results: Vec<usize>,
    pub search_index: usize,
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
    TelegramMessage { chat_id: i64, text: String },
}

pub enum Role {
    User,
    System,
    Assistant,
}

pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}
