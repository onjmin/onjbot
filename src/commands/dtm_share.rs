use serenity::all::{CommandInteraction, Context, ResolvedOption, ResolvedValue};
use serenity::builder::{CreateEmbed, CreateInteractionResponseFollowup};

/// `/dtm-share` スラッシュコマンド
///
/// DTMエディタで生成した共有コード（MMLペイロード）を受け取り、
/// Discord Activity（DTM Player）での再生方法を案内するEmbedを投稿する。
pub async fn handle_slash_dtm_share(ctx: &Context, command: &CommandInteraction) {
    // タイムアウト防止のため即座に defer
    if let Err(why) = command.defer(&ctx.http).await {
        eprintln!("dtm-share deferエラー: {:?}", why);
        return;
    }

    // `code` 引数を取得
    let options = command.data.options();
    let code = if let Some(ResolvedOption {
        value: ResolvedValue::String(s),
        ..
    }) = options.first()
    {
        s.trim().to_string()
    } else {
        let _ = command
            .create_followup(
                &ctx.http,
                (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                    m.content("共有コードを指定してください。").ephemeral(true)
                })(CreateInteractionResponseFollowup::default()),
            )
            .await;
        return;
    };

    // コードの簡易バリデーション（g.xxx / z.xxx / u.xxx 形式）
    let is_valid = code.starts_with("g.") || code.starts_with("z.") || code.starts_with("u.");
    if !is_valid {
        let _ = command
            .create_followup(
                &ctx.http,
                (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                    m.content("無効なコードです。DTMエディタの「🎮 DISCORD」ボタンで生成したコードを貼り付けてください。")
                        .ephemeral(true)
                })(CreateInteractionResponseFollowup::default()),
            )
            .await;
        return;
    }

    // Embed作成
    let embed = CreateEmbed::default()
        .title("🎵 DTM曲を共有")
        .description(format!(
            "ボイスチャンネルで **DTM Player** アクティビティを起動して、\n\
             以下のコードをプレイヤーに貼り付けると再生できます！\n\n\
             ```\n{}\n```",
            code
        ))
        .color(0x29adff_u32) // pico-cyan
        .field(
            "🎮 使い方",
            "1. ボイスチャンネルに参加\n\
             2. 「アクティビティ」→「DTM Player」を起動\n\
             3. 上のコードをコピーして貼り付け → ▶ PLAY",
            false,
        )
        .field(
            "🔗 エディタ",
            "[DTMエディタを開く](https://onjmin.github.io/dtm/demo/index.html)",
            true,
        )
        .footer(serenity::builder::CreateEmbedFooter::new(
            "@onjmin/dtm · Discord Activity",
        ));

    if let Err(why) = command
        .create_followup(
            &ctx.http,
            (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                m.embed(embed)
            })(CreateInteractionResponseFollowup::default()),
        )
        .await
    {
        eprintln!("dtm-share 投稿エラー: {:?}", why);
    }
}
