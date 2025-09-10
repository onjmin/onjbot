use crate::unj::{self, ApiMessage};
use serenity::all::{Context, Message};
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

    // タイピングインジケータを開始
    let typing = msg.channel_id.start_typing(&ctx.http);

    // LLMに問い合わせ、結果を処理
    match crate::llm::talk_to_llama(input).await {
        Ok(response) => {
            // APIに結果をPOST
            let api_message = ApiMessage {
                thread_id,
                cc_user_id: "AI", // CCユーザーをAIとして設定
                cc_user_name: "解音ゼロ",
                cc_user_avatar: 102,
                content_type: 1,         // テキストタイプ
                content_text: &response, // LLMの応答を渡す
                content_url: "",
            };

            if let Err(e) = unj::post_res(&api_message).await {
                eprintln!("API送信中にエラーが発生しました: {}", e);
            }
        }
        Err(e) => {
            // エラーをAPIにPOST
            let error_text = format!("エラー: {}", e);
            let api_message = ApiMessage {
                thread_id,
                cc_user_id: "AI",
                cc_user_name: "",
                cc_user_avatar: 0,
                content_type: 1,
                content_text: &error_text,
                content_url: "",
            };
            if let Err(e) = unj::post_res(&api_message).await {
                eprintln!("API送信中にエラーが発生しました: {}", e);
            }
        }
    }

    // `typing`はスコープを抜けると自動で停止
    drop(typing);

    Ok(())
}
