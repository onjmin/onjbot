use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::{
    all::{Command, Context, CreateCommand, EventHandler, Interaction, Message, Ready},
    async_trait,
};

use crate::commands::{
    chat::handle_message_ai_command, rss::handle_rss_command, rss_random::handle_rss_random_command,
};

pub struct Handler;

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
                _ => {}
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        // Bot自身のメッセージは無視
        if msg.author.bot {
            return;
        }

        // メッセージが "!ai" で始まる（後にスペースや改行含む）
        if msg.content.starts_with("!ai") {
            // "!ai" の部分を取り除き、前後の空白と改行をトリムする
            let user_input = msg.content["!ai".len()..].trim();

            // メッセージ内の改行や複数行もそのまま user_input に含まれる

            // 応答でメンションを飛ばすようにハンドラー呼び出し
            handle_message_ai_command(&ctx, &msg, user_input).await;
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} としてログイン完了！", ready.user.name);

        let builders = vec![
            CreateCommand::new("ping").description("Botが動いているか確認します"),
            CreateCommand::new("rss").description("チャンネルのRSSフィードを投稿します"),
            CreateCommand::new("rss-random").description("ランダムにRSSフィードを投稿します"),
        ];

        let commands = Command::set_global_commands(&ctx.http, builders).await;

        match commands {
            Ok(cmds) => println!("スラッシュコマンド登録完了: {:?}", cmds),
            Err(why) => eprintln!("スラッシュコマンド登録失敗: {:?}", why),
        }
    }
}
