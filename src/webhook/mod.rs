// src/webhook/mod.rs

pub mod ai;
pub mod beep;
pub mod image_gen;

use serenity::model::prelude::Message;
use serenity::prelude::Context;
use sha2::{Digest, Sha256};

use lazy_static::lazy_static;
use serenity::model::prelude::ReactionType;
use std::env;

lazy_static! {
    // lazy_staticで定義されたグローバル変数は、String型への参照として扱われます
    pub static ref UNJ_AI_WEBHOOK_SECRET_PEPPER: String = {
        env::var("UNJ_AI_WEBHOOK_SECRET_PEPPER")
            .expect("UNJ_AI_WEBHOOK_SECRET_PEPPER must be set")
    };
}

/// Webhookメッセージの種類に応じて適切なハンドラに振り分ける
pub async fn handle_webhook_message(ctx: &Context, msg: &Message) {
    let content = msg.content.trim();
    if !content.starts_with("```") || !content.ends_with("```") {
        return;
    }

    let inner_content = &content[3..content.len() - 3].trim();
    let lines: Vec<&str> = inner_content.split('\n').collect();

    // 必要な行数が揃っているか確認
    if lines.len() < 5 {
        eprintln!("Webhookメッセージの行数が不足しています。");
        return;
    }

    // 2行目のハッシュを取得
    let received_hash = lines[1];

    // 5行目以降の全行を結合してユーザー入力テキストを再構築
    let input = lines[4..].join("\n");

    // ハッシュの検証
    if !verify_hash(received_hash, &input) {
        eprintln!("ハッシュ検証に失敗しました。不正なWebhookの可能性があります。");
        // ハッシュが一致しない場合は処理を中断
        if let Err(why) = msg
            .react(&ctx.http, ReactionType::Unicode("❌".to_string()))
            .await
        {
            eprintln!("リアクションを追加できませんでした: {:?}", why);
        }

        return;
    }

    // 検証成功後、コマンドをパースして振り分け
    if input.starts_with("!beep") {
        // 処理開始を示す✅リアクションを追加
        if let Err(why) = msg.react(&ctx.http, '✅').await {
            eprintln!("リアクションを追加できませんでした: {:?}", why);
        }
        let cleaned_input = input.strip_prefix("!beep").unwrap_or(&input).trim();
        if let Err(e) =
            beep::handle_beep_webhook(ctx, msg, &lines[2], &lines[3], cleaned_input).await
        {
            eprintln!("beep Webhookの処理中にエラー: {}", e);
        }
    } else if input.starts_with("!ai") {
        // 処理開始を示す🤖リアクションを追加
        if let Err(why) = msg.react(&ctx.http, '🤖').await {
            eprintln!("リアクションを追加できませんでした: {:?}", why);
        }
        let cleaned_input = input.strip_prefix("!ai").unwrap_or(&input).trim();
        if let Err(e) = ai::handle_ai_webhook(ctx, msg, &lines[2], &lines[3], cleaned_input).await {
            eprintln!("AI Webhookの処理中にエラー: {}", e);
        }
    } else if input.starts_with("!gen") {
        // 処理開始を示す🎨リアクションを追加
        if let Err(why) = msg
            .react(&ctx.http, ReactionType::Unicode("🎨".to_string()))
            .await
        {
            eprintln!("リアクションを追加できませんでした: {:?}", why);
        }
        let cleaned_input = input.strip_prefix("!gen").unwrap_or(&input).trim();
        if let Err(e) =
            image_gen::handle_image_gen_webhook(ctx, msg, &lines[2], &lines[3], cleaned_input).await
        {
            eprintln!("画像生成 Webhookの処理中にエラー: {}", e);
        }
    } else {
        println!("不明なコマンド形式のWebhookを受信しました。");
    }
}

/// JavaScriptのハッシュ生成ロジックをRustで再現
fn verify_hash(received_hash: &str, input: &str) -> bool {
    let first_hash = Sha256::digest(input.as_bytes());
    let first_hash_str = hex::encode(first_hash);

    let mut hasher = Sha256::new();
    let delimiter = "###";

    // ここを修正します。`*`で参照を外して、String型を渡します。
    let combined_string = format!(
        "{}{}{}",
        *UNJ_AI_WEBHOOK_SECRET_PEPPER, delimiter, first_hash_str
    );

    hasher.update(combined_string.as_bytes());
    let final_hash = hasher.finalize();
    let calculated_hash_full = hex::encode(final_hash);
    let calculated_hash = &calculated_hash_full[..8];

    received_hash == calculated_hash
}
