use crate::core::types::AppEvent;
use teloxide::prelude::*;

pub async fn start_telegram_bot(tx: tokio::sync::mpsc::UnboundedSender<AppEvent>) {
    let bot = Bot::from_env();

    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let tx = tx.clone();
        async move {
            let chat_id = msg.chat.id.0;
            let text = msg.text().unwrap_or_default().to_string();
            
            let _ = tx.send(AppEvent::TelegramMessage { chat_id, text });

            respond(())
        }
    })
    .await;
}
