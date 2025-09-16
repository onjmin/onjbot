pub mod ai;
pub mod beep;
pub mod image_gen;

use serenity::model::prelude::Message;
use serenity::prelude::Context;
use sha2::{Digest, Sha256};

use lazy_static::lazy_static;
use serenity::model::prelude::ReactionType;
use std::env;

use dashmap::DashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// ノンスを保持する最大件数
const MAX_NONCE_COUNT: usize = 10_000;

lazy_static! {
    pub static ref UNJ_AI_WEBHOOK_SECRET_PEPPER: String = {
        env::var("UNJ_AI_WEBHOOK_SECRET_PEPPER")
            .expect("UNJ_AI_WEBHOOK_SECRET_PEPPER must be set")
    };

    // DashMapを使い、ノンスと挿入時刻を保存
    //これにより、スレッドセーフな高速アクセスとクリーンアップが可能
    pub static ref USED_NONCES: DashMap<String, u64> = {
        DashMap::new()
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

    if lines.len() != 6 {
        eprintln!("Webhookメッセージの行数の不一致");
        return;
    }

    let received_hash = lines[1];
    let nonce = lines[2];
    let input = lines[5..].join("\n");

    if !verify_hash(received_hash, nonce, &input) {
        eprintln!("ハッシュ検証に失敗しました。不正なWebhookの可能性があります。");
        if let Err(why) = msg
            .react(&ctx.http, ReactionType::Unicode("❌".to_string()))
            .await
        {
            eprintln!("リアクションを追加できませんでした: {:?}", why);
        }
        return;
    }

    // ノンスの再利用をチェック
    if USED_NONCES.contains_key(nonce) {
        eprintln!("警告: nonceが再利用されました。再生攻撃の可能性があります。");
        if let Err(why) = msg
            .react(&ctx.http, ReactionType::Unicode("❌".to_string()))
            .await
        {
            eprintln!("リアクションを追加できませんでした: {:?}", why);
        }
        return;
    }

    // マップのサイズが上限を超えた場合、最も古いノンスを削除
    if USED_NONCES.len() >= MAX_NONCE_COUNT {
        let mut oldest_nonce: Option<String> = None;
        let mut oldest_timestamp = u64::MAX;

        // マップをイテレートして最も古いノンスを特定
        for entry in USED_NONCES.iter() {
            let timestamp = *entry.value();
            if timestamp < oldest_timestamp {
                oldest_timestamp = timestamp;
                oldest_nonce = Some(entry.key().clone());
            }
        }

        // 最も古いノンスを削除
        if let Some(nonce_to_remove) = oldest_nonce {
            USED_NONCES.remove(&nonce_to_remove);
            eprintln!("古いノンスを削除しました: {}", nonce_to_remove);
        }
    }

    // 現在のノンスをマップに追加
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    USED_NONCES.insert(nonce.to_string(), now);

    // 検証成功後、コマンドをパースして振り分け
    if input.starts_with("!beep") {
        if let Err(why) = msg.react(&ctx.http, '✅').await {
            eprintln!("リアクションを追加できませんでした: {:?}", why);
        }
        let cleaned_input = input.strip_prefix("!beep").unwrap_or(&input).trim();
        if let Err(e) =
            beep::handle_beep_webhook(ctx, msg, &lines[3], &lines[4], cleaned_input).await
        {
            eprintln!("beep Webhookの処理中にエラー: {}", e);
        }
    } else if input.starts_with("!ai") {
        if let Err(why) = msg.react(&ctx.http, '🤖').await {
            eprintln!("リアクションを追加できませんでした: {:?}", why);
        }
        let cleaned_input = input.strip_prefix("!ai").unwrap_or(&input).trim();
        if let Err(e) = ai::handle_ai_webhook(ctx, msg, &lines[3], &lines[4], cleaned_input).await {
            eprintln!("AI Webhookの処理中にエラー: {}", e);
        }
    } else if input.starts_with("!gen") {
        if let Err(why) = msg
            .react(&ctx.http, ReactionType::Unicode("🎨".to_string()))
            .await
        {
            eprintln!("リアクションを追加できませんでした: {:?}", why);
        }
        let cleaned_input = input.strip_prefix("!gen").unwrap_or(&input).trim();
        if let Err(e) =
            image_gen::handle_image_gen_webhook(ctx, msg, &lines[3], &lines[4], cleaned_input).await
        {
            eprintln!("画像生成 Webhookの処理中にエラー: {}", e);
        }
    } else {
        println!("不明なコマンド形式のWebhookを受信しました。");
    }
}

fn verify_hash(received_hash: &str, nonce: &str, input: &str) -> bool {
    let input_hash = Sha256::digest(input.as_bytes());
    let input_hash_str = hex::encode(input_hash);

    let delimiter = "###";
    let combined_string = format!(
        "{}{}{}{}{}",
        *UNJ_AI_WEBHOOK_SECRET_PEPPER, delimiter, input_hash_str, delimiter, nonce
    );

    let final_hash = Sha256::digest(combined_string.as_bytes());
    let calculated_hash_full = hex::encode(final_hash);
    let calculated_hash = &calculated_hash_full[..8];

    received_hash == calculated_hash
}
