use serenity::{
    all::{Command, Context, CreateCommand, EventHandler, Interaction, Ready},
    async_trait,
};

use crate::commands::{rss::handle_rss_command, rss_random::handle_rss_random_command};

pub struct Handler;

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

        let builders = vec![
            CreateCommand::new("rss").description("チャンネルのRSSを投稿します"),
            CreateCommand::new("rss-random").description("ランダムにRSSを投稿します"),
        ];

        let commands = Command::set_global_commands(&ctx.http, builders).await;

        match commands {
            Ok(cmds) => println!("スラッシュコマンド登録完了: {:?}", cmds),
            Err(why) => eprintln!("スラッシュコマンド登録失敗: {:?}", why),
        }
    }
}
