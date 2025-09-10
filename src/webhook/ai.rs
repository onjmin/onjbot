// src/webhook/ai.rs
use serenity::model::prelude::Message;
use serenity::prelude::Context;
use std::error::Error;

/// AI関連のWebhookメッセージを処理するハンドラ
pub async fn handle_ai_webhook(
    ctx: &Context,
    msg: &Message,
    thread_id: &str,
    res_count: &str,
    input: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("AI Webhookを受信しました。");
    println!("  スレッドID: {}", thread_id);
    println!("  レス番号: {}", res_count);
    println!("  入力テキスト: {}", input);

    // TODO: ここに実際のAI処理ロジックを実装します

    // 処理が完了したことを示すリアクションを追加
    // 例として、✅の絵文字をリアクションとして追加します。
    // メッセージが正常に処理されたことをユーザーに知らせることができます。
    if let Err(why) = msg.react(&ctx.http, '✅').await {
        eprintln!("リアクションを追加できませんでした: {:?}", why);
    }

    // 正常終了
    Ok(())
}
