use serenity::all::CommandDataOptionValue;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::{
    all::{Command, Context, CreateCommand, EventHandler, Interaction, Ready},
    async_trait,
};

use crate::commands::{
    chat::handle_chat_command, rss::handle_rss_command, rss_random::handle_rss_random_command,
};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Some(command) = interaction.command() {
            match command.data.name.as_str() {
                "ai" => {
                    let user_input = match command.data.options.get(0).map(|opt| &opt.value) {
                        Some(CommandDataOptionValue::String(s)) => s.as_str(),
                        _ => "",
                    };
                    handle_chat_command(&ctx, &command, user_input).await;
                }
                "rss" => {
                    handle_rss_command(&ctx, &command).await;
                }
                "rss-random" => {
                    handle_rss_random_command(&ctx, &command).await;
                }
                "ping" => {
                    let builder = CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new().content("pong"),
                    );
                    let _ = command.create_response(&ctx.http, builder).await;
                }
                _ => {}
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} としてログイン完了！", ready.user.name);

        let builders = vec![
            CreateCommand::new("ai").description("解音ロゼと会話できます"),
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
