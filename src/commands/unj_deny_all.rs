use crate::unj::{get_deny_all, post_deny_all};
use serenity::all::{
    CommandDataOptionValue, CommandInteraction, Context, CreateInteractionResponseFollowup,
};
use std::env;

pub async fn handle_slash_unj_deny_all(ctx: &Context, command: &CommandInteraction) {
    if let Err(e) = command.defer_ephemeral(&ctx.http).await {
        eprintln!("defer エラー: {:?}", e);
        return;
    }
    // モデレーターチェック
    let mod_role_id: u64 = env::var("DISCORD_MODERATOR_ROLE_ID")
        .expect("DISCORD_MODERATOR_ROLE_ID must be set")
        .parse()
        .expect("DISCORD_MODERATOR_ROLE_ID must be a number");
    let mod_role = serenity::all::RoleId::new(mod_role_id);

    let member = match command.member.as_ref() {
        Some(m) => m,
        None => {
            let _ = command
                .create_followup(
                    &ctx.http,
                    (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                        m.content("❌ ユーザー情報を取得できませんでした")
                            .ephemeral(true)
                    })(CreateInteractionResponseFollowup::default()),
                )
                .await;
            return;
        }
    };

    if !member.roles.iter().any(|r| *r == mod_role) {
        let _ = command
            .create_followup(
                &ctx.http,
                (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                    m.content("❌ このコマンドを実行できるのはモデレーターだけです")
                        .ephemeral(true)
                })(CreateInteractionResponseFollowup::default()),
            )
            .await;
        return;
    }

    // 引数解析
    let arg = command
        .data
        .options
        .get(0)
        .and_then(|opt| match opt.value {
            CommandDataOptionValue::Boolean(b) => Some(b),
            _ => None,
        })
        .map(|b| b.to_string())
        .unwrap_or_default();

    let content = if arg == "true" {
        match post_deny_all(true).await {
            Ok(res) => format!("✅ denyAll を {} に設定しました", res.deny_all),
            Err(e) => format!("❌ 設定失敗: {}", e),
        }
    } else if arg == "false" {
        match post_deny_all(false).await {
            Ok(res) => format!("✅ denyAll を {} に設定しました", res.deny_all),
            Err(e) => format!("❌ 設定失敗: {}", e),
        }
    } else {
        match get_deny_all().await {
            Ok(res) => format!("現在の denyAll 状態: {}", res.deny_all),
            Err(e) => format!("❌ 取得失敗: {}", e),
        }
    };

    let _ = command
        .create_followup(
            &ctx.http,
            (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                m.content(content).ephemeral(true) // 必要に応じて true/false
            })(CreateInteractionResponseFollowup::default()),
        )
        .await;
}
