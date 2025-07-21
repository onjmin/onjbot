use serenity::{
    all::{
        ChannelId, Client, Command, CommandInteraction, Context, EventHandler, GatewayIntents,
        Interaction, Ready,
    },
    async_trait,
    builder::{CreateCommand, CreateInteractionResponseFollowup},
};

use dotenvy::dotenv;
use std::{
    collections::HashSet,
    env,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use feed_rs::parser;
use once_cell::sync::Lazy;
use rand::rngs::StdRng;
use rand::{SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha20Rng;
use reqwest;
use tokio::sync::Mutex;

static FEEDS: Lazy<Arc<Mutex<Vec<(ChannelId, String)>>>> = Lazy::new(|| {
    Arc::new(Mutex::new(vec![
        (
            ChannelId::new(1396030037362216980),
            "https://qiita.com/tags/python/feed".to_string(),
        ),
        (
            ChannelId::new(1396051371580461148),
            "https://zenn.dev/topics/scratch/feed".to_string(),
        ),
    ]))
});

static POSTED_URLS: Lazy<Arc<Mutex<HashSet<String>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashSet::new())));

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Some(command) = interaction.command() {
            match command.data.name.as_str() {
                "rss" => {
                    handle_rss_command(&ctx, &command).await;
                }
                "rss-random" => {
                    handle_rss_random_command(&ctx, &command).await;
                }
                _ => {}
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} としてログイン完了！", ready.user.name);

        // Vec<CreateCommand> を直接作って渡します
        let builders = vec![
            CreateCommand::new("rss").description("チャンネルのRSSを投稿します"),
            CreateCommand::new("rss-random").description("ランダムにRSSを投稿します"),
        ];

        // serenity::all::Command を使って登録
        let commands = Command::set_global_commands(&ctx.http, builders).await;

        match commands {
            Ok(cmds) => println!("スラッシュコマンド登録完了: {:?}", cmds),
            Err(why) => eprintln!("スラッシュコマンド登録失敗: {:?}", why),
        }
    }
}

async fn handle_rss_command(ctx: &Context, command: &CommandInteraction) {
    if let Err(why) = command.defer_ephemeral(&ctx.http).await {
        eprintln!("deferエラー: {:?}", why);
        return;
    }

    let channel_id = command.channel_id;

    let feed_url = {
        let feeds = FEEDS.lock().await;
        feeds
            .iter()
            .find(|(id, _)| *id == channel_id)
            .map(|(_, url)| url.clone())
    };

    let feed_url = match feed_url {
        Some(url) => url,
        None => {
            let _ = command
                .create_followup(
                    &ctx.http,
                    (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                        m.content("このチャンネルに紐づくRSSフィードが登録されていません。")
                            .ephemeral(true)
                    })(CreateInteractionResponseFollowup::default()),
                ) // ←ここでクロージャを呼び出して値を作る
                .await;
            return;
        }
    };

    match fetch_and_post_rss(ctx, channel_id, &feed_url).await {
        Ok(_) => {
            let _ = command
                .create_followup(
                    &ctx.http,
                    (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                        m.content("RSS記事を投稿しました。").ephemeral(true)
                    })(CreateInteractionResponseFollowup::default()),
                ) // ←ここでクロージャを呼び出して値を作る
                .await;
        }
        Err(e) => {
            let _ = command
                .create_followup(
                    &ctx.http,
                    (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                        m.content(format!("RSS取得中にエラーが発生しました: {}", e))
                            .ephemeral(true)
                    })(CreateInteractionResponseFollowup::default()),
                )
                .await;
        }
    }
}

async fn handle_rss_random_command(ctx: &Context, command: &CommandInteraction) {
    if let Err(why) = command.defer_ephemeral(&ctx.http).await {
        eprintln!("deferエラー: {:?}", why);
        return;
    }

    let (channel_id, feed_url) = {
        let feeds = FEEDS.lock().await;
        let mut rng = {
            let duration_since_epoch = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            ChaCha20Rng::seed_from_u64(duration_since_epoch.as_secs())
        };
        match feeds.choose(&mut rng) {
            Some((id, url)) => (id.clone(), url.clone()),
            None => {
                let _ = command
                .create_followup(
                    &ctx.http,
                    (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                        m.content("RSSフィードが登録されていません。").ephemeral(true)
                    })(CreateInteractionResponseFollowup::default()),
                ) // ←ここでクロージャを呼び出して値を作る
                .await;
                return;
            }
        }
    };

    match fetch_and_post_rss(ctx, channel_id, &feed_url).await {
        Ok(_) => {
            let _ = command
                .create_followup(
                    &ctx.http,
                    (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                        m.content("RSS記事を投稿しました。").ephemeral(true)
                    })(CreateInteractionResponseFollowup::default()),
                ) // ←ここでクロージャを呼び出して値を作る
                .await;
        }
        Err(e) => {
            let _ = command
                .create_followup(
                    &ctx.http,
                    (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                        m.content(format!("RSS取得中にエラーが発生しました: {}", e))
                            .ephemeral(true)
                    })(CreateInteractionResponseFollowup::default()),
                ) // ←ここでクロージャを呼び出して値を作る
                .await;
        }
    }
}

async fn fetch_and_post_rss(
    ctx: &Context,
    channel_id: ChannelId,
    feed_url: &str,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (compatible; MyRustBot/1.0)")
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .get(feed_url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let text = resp.text().await.map_err(|e| e.to_string())?;

    let feed = parser::parse(text.as_bytes()).map_err(|e| e.to_string())?;

    // 記事が存在するか確認
    if feed.entries.is_empty() {
        return Err("記事がありません".to_string());
    }

    // ランダムに選ぶ（スレッド安全な乱数生成器を使用）
    let mut rng = StdRng::from_entropy();
    let entry = feed
        .entries
        .choose(&mut rng)
        .ok_or("記事が選べませんでした")?;
    let link = entry
        .links
        .first()
        .ok_or("リンクがありません")?
        .href
        .clone();

    {
        let mut posted = POSTED_URLS.lock().await;
        if posted.contains(&link) {
            return Err("（すでに投稿済みの記事です）".to_string());
        }
        posted.insert(link.clone());
    }

    channel_id
        .say(&ctx.http, format!("新着記事: {}", link))
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
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
