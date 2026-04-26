use crate::core::types::ChatMessage;

pub fn estimate_tokens(messages: &[ChatMessage]) -> usize {
    let mut total_chars = 0;
    for msg in messages {
        total_chars += msg.content.len();
    }
    total_chars / 4
}