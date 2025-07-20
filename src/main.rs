use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};
use dotenvy::dotenv;
use std::env;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                eprintln!("エラー: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} としてログイン完了！", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok(); // .env 読み込み
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN が環境変数にありません");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Botのクライアント作成に失敗しました");

    if let Err(why) = client.start().await {
        eprintln!("Bot起動エラー: {:?}", why);
    }
}
