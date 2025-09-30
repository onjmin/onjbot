use serenity::all::{CommandInteraction, Context};
use serenity::builder::CreateInteractionResponseFollowup;

use crate::{rss::fetch_and_post_rss, state::FEEDS};

pub async fn handle_slash_rss(ctx: &Context, command: &CommandInteraction) {
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
                )
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
