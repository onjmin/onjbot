use crate::commands::{
    chat::handle_message_ai_command, rss::handle_rss_command, rss_random::handle_rss_random_command,
};
use crate::webhook;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::{
    all::{Command, Context, CreateCommand, EventHandler, Interaction, Message, Ready},
    async_trait,
};
pub struct Handler;
use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;
use serenity::model::id::UserId;
use std::sync::Mutex;

static ZENRES_STATE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static BOT_USER_ID: OnceCell<UserId> = OnceCell::new();

use serenity::model::id::ChannelId;
use std::env;

static TARGET_CHANNEL_ID: Lazy<ChannelId> = Lazy::new(|| {
    let channel_id_str = env::var("UNJ_AI_WEBHOOK_CHANNEL_ID")
        .expect("UNJ_AI_WEBHOOK_CHANNEL_ID が環境変数にありません");
    let channel_id_u64 = channel_id_str
        .parse::<u64>()
        .expect("UNJ_AI_WEBHOOK_CHANNEL_ID の値が不正な数値です");
    ChannelId::new(channel_id_u64)
});

#[async_trait]
impl EventHandler for Handler {
    // 既存のスラッシュコマンド処理はそのまま
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Some(command) = interaction.command() {
            match command.data.name.as_str() {
                "ping" => {
                    let builder = CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().content("pong"),
                    );
                    let _ = command.create_response(&ctx.http, builder).await;
                }
                "rss" => {
                    handle_rss_command(&ctx, &command).await;
                }
                "rss-random" => {
                    handle_rss_random_command(&ctx, &command).await;
                }
                "zenres" => {
                    let status = {
                        let mut state = ZENRES_STATE.lock().unwrap();
                        *state = !*state;
                        if *state { "ON" } else { "OFF" }.to_string()
                    };

                    let builder = CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content(format!("全レスモード: {}", status)),
                    );
                    let _ = command.create_response(&ctx.http, builder).await;
                }
                _ => {}
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // 自分自身のメッセージだけ無視する
        if let Some(my_id) = BOT_USER_ID.get() {
            if msg.author.id == *my_id {
                return;
            }
        }

        // 全レスモード
        let zenres = *ZENRES_STATE.lock().unwrap();

        if zenres {
            let content = msg
                .content
                .strip_prefix("!ai")
                .map(str::trim)
                .unwrap_or_else(|| msg.content.trim());

            handle_message_ai_command(&ctx, &msg, content).await;
        } else if msg.content.starts_with("!ai") {
            let user_input = msg.content["!ai".len()..].trim();
            handle_message_ai_command(&ctx, &msg, user_input).await;
        }

        // うんJ AI Webhook監視
        if msg.channel_id == *TARGET_CHANNEL_ID {
            webhook::handle_webhook_message(&ctx, &msg).await;
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} としてログイン完了！", ready.user.name);
        BOT_USER_ID.set(ready.user.id).ok();

        let builders = vec![
            CreateCommand::new("ping").description("Botが動いているか確認します"),
            CreateCommand::new("rss").description("チャンネルのRSSフィードを投稿します"),
            CreateCommand::new("rss-random").description("ランダムにRSSフィードを投稿します"),
            CreateCommand::new("zenres")
                .description("全てのメッセージに反応するモードに切り替えます"),
        ];

        let commands = Command::set_global_commands(&ctx.http, builders).await;

        match commands {
            Ok(cmds) => println!("スラッシュコマンド登録完了: {:?}", cmds),
            Err(why) => eprintln!("スラッシュコマンド登録失敗: {:?}", why),
        }
    }
}
