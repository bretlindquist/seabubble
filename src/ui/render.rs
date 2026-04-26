use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};
use crate::core::types::ChatMessage;

pub fn render_markdown(text: &str) -> Vec<Line> {
    let mut lines = Vec::new();
    for line in text.lines() {
        let mut spans = Vec::new();
        let parts: Vec<&str> = line.split('`').collect();
        for (i, part) in parts.into_iter().enumerate() {
            if i % 2 == 1 {
                // Inside backticks
                spans.push(Span::styled(part.to_string(), Style::default().fg(Color::Green)));
            } else {
                // Outside backticks
                spans.push(Span::raw(part.to_string()));
            }
        }
        lines.push(Line::from(spans));
    }
    lines
}

pub fn draw_ui<B: Backend>(frame: &mut Frame, messages: &[ChatMessage]) {
    // 2. Modify `draw_ui` to split the `frame.size()` into 3 vertical chunks:
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(1),    // Chat
            Constraint::Length(1), // Footer
        ])
        .split(frame.size());

    // 3. Header: Create a simple Paragraph with white text on a blue background: ` 🐢 SeaTurtle V2 `
    let header = Paragraph::new(" 🐢 SeaTurtle V2 ")
        .style(Style::default().fg(Color::White).bg(Color::Blue));
    frame.render_widget(header, chunks[0]);

    // 5. Chat window with messages using render_markdown
    let mut chat_text = Vec::new();
    for msg in messages {
        chat_text.extend(render_markdown(&msg.content));
    }
    let chat = Paragraph::new(chat_text).block(Block::default()); // No borders
    frame.render_widget(chat, chunks[1]);

    // 4. Footer: Change the input widget. Single line at the bottom. 
    // It should look like ` [NORMAL] > input_buffer_here ` or ` [STREAMING] ... `.
    // Mocking the input state for conceptual compilation
    let is_streaming = false; // Example state
    let input_text = if is_streaming {
        " [STREAMING] ... ".to_string()
    } else {
        format!(" [NORMAL] > {} ", "input_buffer_here")
    };
    
    let footer = Paragraph::new(input_text);
    frame.render_widget(footer, chunks[2]);
}
