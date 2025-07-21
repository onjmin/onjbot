use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, id::ChannelId},
    prelude::*,
};
use dotenvy::dotenv;
use std::{collections::HashSet, env, sync::Arc};

use once_cell::sync::Lazy;
use rand_chacha::ChaCha20Rng;
use rand::SeedableRng;
use rand::seq::SliceRandom;
use rss::Channel;
use tokio::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static FEEDS: Lazy<Arc<Mutex<Vec<(ChannelId, String)>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(vec![
        (ChannelId::new(1396030037362216980), "https://qiita.com/tags/python/feed".to_string()), // python
        (ChannelId::new(1396051371580461148), "https://zenn.dev/topics/scratch/feed".to_string()), // scratch
    ]))
});

static POSTED_URLS: Lazy<Arc<Mutex<HashSet<String>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(HashSet::new()))
});

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                eprintln!("エラー: {:?}", why);
            }
            return;
        }

        // !rss → 今のチャンネルに最新記事を投稿
        if msg.content == "!rss" {
            let channel_id = msg.channel_id;

            // フィードを登録済みからランダムに選択
            let (feed_url, ) = {
                let feeds = FEEDS.lock().await;
                if feeds.is_empty() {
                    let _ = channel_id.say(&ctx.http, "RSSフィードが登録されていません").await;
                    return;
                }
                // ランダムに1つだけURLを選ぶ（チャンネルは無視）
                let urls: Vec<String> = feeds.iter().map(|(_, url)| url.clone()).collect();
                let mut rng = {
                    let duration_since_epoch = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap();
                    ChaCha20Rng::seed_from_u64(duration_since_epoch.as_secs())
                };
                match urls.choose(&mut rng) {
                    Some(url) => (url.clone(),),
                    None => {
                        let _ = channel_id.say(&ctx.http, "RSSフィードが登録されていません").await;
                        return;
                    }
                }
            };

            // 以下は共通処理
            send_rss_article(&ctx, channel_id, feed_url.clone()).await;
            return;
        }

        // !rss-random → ランダムなチャンネルに最新記事を投稿
        if msg.content == "!rss-random" {
            // チャンネルとURLのペアをランダム選択
            let (channel_id, url) = {
                let feeds = FEEDS.lock().await;
                if feeds.is_empty() {
                    let _ = msg.channel_id.say(&ctx.http, "RSSフィードが登録されていません").await;
                    return;
                }
                let mut rng = {
                    let duration_since_epoch = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap();
                    ChaCha20Rng::seed_from_u64(duration_since_epoch.as_secs())
                };
                match feeds.choose(&mut rng) {
                    Some(pair) => (pair.0.clone(), pair.1.clone()),
                    None => {
                        let _ = msg.channel_id.say(&ctx.http, "RSSフィードが登録されていません").await;
                        return;
                    }
                }
            };

            send_rss_article(&ctx, channel_id, url).await;
            return;
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} としてログイン完了！", ready.user.name);
    }
}

/// RSSフィードから最新記事を取り出して指定チャンネルに投稿する関数
async fn send_rss_article(ctx: &Context, channel_id: ChannelId, url: String) {
    let resp = match reqwest::get(&url).await {
        Ok(r) => r,
        Err(_) => {
            let _ = channel_id.say(&ctx.http, "RSSの取得に失敗しました").await;
            return;
        }
    };

    let text = match resp.text().await {
        Ok(t) => t,
        Err(_) => return,
    };

    let channel = match Channel::read_from(text.as_bytes()) {
        Ok(c) => c,
        Err(_) => return,
    };

    let item = match channel.items().first() {
        Some(i) => i,
        None => {
            let _ = channel_id.say(&ctx.http, "記事がありませんでした").await;
            return;
        }
    };

    let link = match item.link() {
        Some(l) => l,
        None => return,
    };

    // 投稿済みチェック
    let already_posted = {
        let posted = POSTED_URLS.lock().await;
        posted.contains(link)
    };

    if already_posted {
        let _ = channel_id.say(&ctx.http, "（すでに投稿済みの記事です）").await;
        return;
    }

    // 投稿済みセットに追加
    {
        let mut posted = POSTED_URLS.lock().await;
        posted.insert(link.to_string());
    }

    let _ = channel_id.say(&ctx.http, format!("新着記事: {}", link)).await;
}

#[tokio::main]
async fn main() {
    dotenv().ok();
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
