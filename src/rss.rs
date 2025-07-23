use crate::state::POSTED_URLS;
use feed_rs::parser;
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use serenity::all::{ChannelId, Context};

pub async fn fetch_and_post_rss(
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

    if feed.entries.is_empty() {
        return Err("記事がありません".to_string());
    }

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
