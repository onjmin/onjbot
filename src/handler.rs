use crate::commands::{
    ai::handle_message_ai_command, image_gen::handle_image_gen_command, rss::handle_rss_command,
    rss_random::handle_rss_random_command,
};
use crate::webhook;
use once_cell::sync::Lazy;
use once_cell::sync::OnceCell;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::id::ChannelId;
use serenity::model::id::UserId;
use serenity::{
    all::{
        Command, CommandOptionType, Context, CreateCommand, CreateCommandOption, EventHandler,
        Interaction, Message, Ready,
    },
    async_trait,
};
use std::sync::Mutex;

// Handler構造体を修正
pub struct Handler {
    pub target_channel_id: ChannelId,
}

static ZENRES_STATE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(false));
static BOT_USER_ID: OnceCell<UserId> = OnceCell::new();

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
                "gen" => {
                    handle_image_gen_command(&ctx, &command).await;
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
        if msg.channel_id == self.target_channel_id {
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
            CreateCommand::new("gen")
                .description("画像生成します")
                .add_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        "prompt", // 引数名
                        "生成したい画像のプロンプト",
                    )
                    .required(true), // プロンプトを必須にする
                ),
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
