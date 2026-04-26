use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::core::types::AppState;

pub fn draw_status(frame: &mut Frame, _state: &AppState) {
    let area = frame.size();

    let text = "SeaTurtle V2 Rust Engine\n\
                Context Mode: Active\n\
                MCP Tools: Stdio/SSE Connected\n\
                Lore Files Loaded: (mock list)";

    let paragraph = Paragraph::new(text)
        .block(Block::default().title("Status").borders(Borders::ALL))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Left);

    frame.render_widget(paragraph, area);
}
