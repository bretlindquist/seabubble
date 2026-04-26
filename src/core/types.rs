pub enum AppMode {
    Normal,
    Insert,
    Search,
    Status,
    Streaming,
    Steering,
    Voice,
    PermissionPrompt(ToolCall),
}

pub struct AppState {
    pub mode: AppMode,
    pub messages: Vec<ChatMessage>,
    pub input_buffer: String,
    pub search_results: Vec<usize>,
    pub search_index: usize,
    pub token_estimate: usize,
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
    TokenReceived(String),
    TelegramMessage { chat_id: i64, text: String },
    ContextWarning,
    HistoryCompacted(Vec<ChatMessage>),
}

#[derive(Clone)]
pub enum Role {
    User,
    System,
    Assistant,
}

#[derive(Clone)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}
