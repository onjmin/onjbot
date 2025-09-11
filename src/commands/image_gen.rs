use base64::{Engine as _, engine::general_purpose::STANDARD};
use serenity::all::{CommandInteraction, Context, ResolvedOption, ResolvedValue};
use serenity::builder::{CreateAttachment, CreateInteractionResponseFollowup};

pub async fn handle_image_gen_command(ctx: &Context, command: &CommandInteraction) {
    // タイムアウトを防ぐため、コマンドを即座に遅延応答させる
    if let Err(why) = command.defer(&ctx.http).await {
        eprintln!("deferエラー: {:?}", why);
        return;
    }

    // ResolvedOptionを直接パターンマッチングします
    let options = command.data.options();
    let user_prompt = if let Some(ResolvedOption {
        value: ResolvedValue::String(prompt),
        ..
    }) = options.first()
    {
        prompt
    } else {
        let _ = command
            .create_followup(
                &ctx.http,
                (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                    m.content("プロンプトを指定してください。").ephemeral(true)
                })(CreateInteractionResponseFollowup::default()),
            )
            .await;
        return;
    };

    // `generate_image_with_sd`関数を呼び出して画像を生成
    let image_data_base64 = match crate::stable_diffusion::generate_image_with_sd(user_prompt).await
    {
        Ok(data) => data,
        Err(e) => {
            let _ = command
                .create_followup(
                    &ctx.http,
                    (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                        m.content(format!("画像生成エラー: {}", e)).ephemeral(true)
                    })(CreateInteractionResponseFollowup::default()),
                )
                .await;
            return;
        }
    };

    // 返されたBase64データをバイナリにデコード
    let image_bytes = match STANDARD.decode(image_data_base64) {
        Ok(bytes) => bytes,
        Err(e) => {
            let _ = command
                .create_followup(
                    &ctx.http,
                    (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                        m.content(format!("画像デコードエラー: {}", e))
                            .ephemeral(true)
                    })(CreateInteractionResponseFollowup::default()),
                )
                .await;
            return;
        }
    };

    // デコードしたバイナリデータからDiscord添付ファイルを作成
    let attachment = CreateAttachment::bytes(image_bytes, "generated_image.png".to_string());

    // 画像をDiscordチャンネルに投稿
    if let Err(why) = command
        .create_followup(
            &ctx.http,
            (|m: CreateInteractionResponseFollowup| -> CreateInteractionResponseFollowup {
                m.content(format!("`{}`の画像です:", user_prompt))
                    .add_file(attachment)
            })(CreateInteractionResponseFollowup::default()),
        )
        .await
    {
        eprintln!("画像投稿エラー: {:?}", why);
    }
}
