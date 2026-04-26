use core::types::{AppMode, AppState, ChatMessage};

mod core;
pub mod mcp;
mod services;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("  __");
    println!(" >(')____, ");
    println!("   (` =~~/  ");
    println!(" ^~^`---'~^~^~");
    println!("SeaTurtle V2 Engine Booting...");
    std::thread::sleep(std::time::Duration::from_millis(600));

    ui::setup_terminal()?;

    let mut state = AppState {
        mode: AppMode::Normal,
        messages: Vec::new(),
        input_buffer: String::new(),
        search_results: Vec::new(),
        search_index: 0,
        token_estimate: 0,
    };

    // Dummy event loop simulation
    // If the user types `/search` in Insert mode and hits `Enter`
    if let AppMode::Insert = state.mode {
        if state.input_buffer.starts_with("/gh-issues") {
            state.mode = AppMode::Streaming;
            state.messages.push(core::types::ChatMessage {
                role: core::types::Role::User,
                content: state.input_buffer.clone(),
            });
            let repo = state.input_buffer.strip_prefix("/gh-issues").unwrap().trim().to_string();
            let (tx, _rx) = tokio::sync::mpsc::unbounded_channel::<core::types::AppEvent>();
            tokio::spawn(async move {
                core::orchestrator::handle_gh_issues(&repo, tx).await;
            });
            state.input_buffer.clear();
        } else if state.input_buffer == "/search" {
            state.mode = AppMode::Search;
            state.input_buffer.clear();
        }
    } else if let AppMode::Search = state.mode {
        // In AppMode::Search, Enter performs case-insensitive substring search of input_buffer across state.messages
        state.search_results.clear();
        let query = state.input_buffer.to_lowercase();
        for (i, msg) in state.messages.iter().enumerate() {
            if msg.content.to_lowercase().contains(&query) {
                state.search_results.push(i);
            }
        }
        state.search_index = 0;
        state.mode = AppMode::Normal;
        println!("Search matches: {}", state.search_results.len());
    } else if let AppMode::Streaming = state.mode {
        // If user hits Backspace while streaming
        let mut _cancel_tx = Some(()); // dummy cancel tx
                                       // send cancel signal
        let _ = _cancel_tx.take();
        state.mode = AppMode::Steering;
        state.input_buffer.clear();
    } else if let AppMode::Steering = state.mode {
        // If user hits Enter while in Steering mode
        let steer = state.input_buffer.clone();
        if let Some(msg) = state
            .messages
            .iter_mut()
            .filter(|m| matches!(m.role, core::types::Role::User))
            .last()
        {
            msg.content.push_str(&format!("\n\n[STEER]: {}", steer));
        }
        if let Some(last) = state.messages.last() {
            if matches!(last.role, core::types::Role::Assistant) {
                state.messages.pop();
            }
        }
        state.mode = AppMode::Streaming;
        // spawn API call
    }

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<core::types::AppEvent>();

    if std::env::var("TELOXIDE_TOKEN").is_ok() {
        let bot_tx = tx.clone();
        tokio::spawn(async move {
            services::telegram::start_telegram_bot(bot_tx).await;
        });
    }

    while let Some(event) = rx.recv().await {
        state.token_estimate = core::context::estimate_tokens(&state.messages);
        if state.token_estimate > 200_000 {
            // Emitting would be an infinite loop if we push to tx immediately without debouncing,
            // but we follow the instruction strictly.
            let _ = tx.send(core::types::AppEvent::ContextWarning);
        }

        match event {
            core::types::AppEvent::ToolCallResult(_) => {}
            core::types::AppEvent::TelegramMessage { chat_id, text } => {
                state.messages.push(core::types::ChatMessage {
                    role: core::types::Role::User,
                    content: format!("[Telegram from {}] {}", chat_id, text),
                });
            }
            core::types::AppEvent::ContextWarning => {
                // Pause stream by changing mode if streaming
                if let AppMode::Streaming = state.mode {
                    state.mode = AppMode::Steering;
                }
                let messages = state.messages.clone();
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    let compacted = core::orchestrator::compact_history(messages).await;
                    let _ = tx_clone.send(core::types::AppEvent::HistoryCompacted(compacted));
                });
            }
            core::types::AppEvent::HistoryCompacted(new_messages) => {
                state.messages = new_messages;
                // Resume stream
                if let AppMode::Steering = state.mode {
                    state.mode = AppMode::Streaming;
                }
            }
        }
    }

    Ok(())
}
