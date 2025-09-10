mod commands;
mod handler;
mod llm;
mod rss;
mod state;
mod unj;
mod webhook;

use dotenvy::dotenv;
use serenity::all::{Client, GatewayIntents};
use std::env;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN が環境変数にありません");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(handler::Handler)
        .await
        .expect("Botのクライアント作成に失敗しました");

    if let Err(why) = client.start().await {
        eprintln!("Bot起動エラー: {:?}", why);
    }
}
