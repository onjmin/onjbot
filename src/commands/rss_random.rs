use serenity::all::{CommandInteraction, Context};
use serenity::builder::CreateInteractionResponseFollowup;
use std::time::{SystemTime, UNIX_EPOCH};

use rand::{SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha20Rng;

use crate::{rss::fetch_and_post_rss, state::FEEDS};

pub async fn handle_slash_rss_random(ctx: &Context, command: &CommandInteraction) {
    if let Err(why) = command.defer_ephemeral(&ctx.http).await {
        eprintln!("deferエラー: {:?}", why);
        return;
    }

    let (channel_id, feed_url) = {
        let feeds = FEEDS.lock().await;
        let mut rng = {
            let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
            ChaCha20Rng::seed_from_u64(duration.as_secs())
        };
        match feeds.choose(&mut rng) {
            Some((id, url)) => (id.clone(), url.clone()),
            None => {
                let _ = command
                .create_followup(
                    &ctx.http,
                    (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                        m.content("RSSフィードが登録されていません。")
                            .ephemeral(true)
                    })(CreateInteractionResponseFollowup::default()),
                )
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
                )
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
