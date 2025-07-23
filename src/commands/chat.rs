use serenity::all::{Context, Message};

pub async fn handle_message_ai_command(ctx: &Context, msg: &Message, user_input: &str) {
    match crate::llm::talk_to_llama(user_input).await {
        Ok(response) => {
            // メンションを付けて返信
            let content = format!("<@{}> {}", msg.author.id, response);
            let _ = msg.channel_id.say(&ctx.http, content).await;
        }
        Err(e) => {
            let content = format!("<@{}> エラー: {}", msg.author.id, e);
            let _ = msg.channel_id.say(&ctx.http, content).await;
        }
    }
}
