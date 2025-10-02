mod commands;
mod feeder;
mod handler;
mod llm;
mod rss;
mod stable_diffusion;
mod state;
mod unj;
mod webhook;

use dotenvy::dotenv;
use serenity::all::{Client, GatewayIntents};
use serenity::model::id::ChannelId;
use std::env;

use handler::Handler;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN が環境変数にありません");

    let target_channel_id_str = env::var("UNJ_AI_WEBHOOK_CHANNEL_ID")
        .expect("UNJ_AI_WEBHOOK_CHANNEL_ID が環境変数にありません");
    let target_channel_id_u64 = target_channel_id_str
        .parse::<u64>()
        .expect("UNJ_AI_WEBHOOK_CHANNEL_ID の値が不正な数値です");
    let target_channel_id = ChannelId::new(target_channel_id_u64);

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler { target_channel_id })
        .await
        .expect("Botのクライアント作成に失敗しました");

    tokio::spawn(async {
        if let Err(e) = feeder::keep_session_alive().await {
            eprintln!("Session keeper task crashed: {:?}", e);
        }
    });

    if let Err(why) = client.start().await {
        eprintln!("Bot起動エラー: {:?}", why);
    }
}
