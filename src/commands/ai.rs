use serenity::all::{Context, Message};

pub async fn handle_slash_ai(ctx: &Context, msg: &Message, user_input: &str) {
    // タイピングインジケータを開始（dropされるまで続く）
    let typing = msg.channel_id.start_typing(&ctx.http);

    match crate::llm::talk_to_llm(user_input).await {
        Ok(response) => {
            let content = format!("<@{}> {}", msg.author.id, response);
            let _ = msg.channel_id.say(&ctx.http, content).await;
        }
        Err(e) => {
            let content = format!("<@{}> エラー: {}", msg.author.id, e);
            let _ = msg.channel_id.say(&ctx.http, content).await;
        }
    }

    // typingはスコープを抜けたときに自動で止まります（Drop実装）
    drop(typing); // 明示的に書いてもOK（なくてもOK）
}
