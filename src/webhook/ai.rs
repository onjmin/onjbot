use crate::unj::{self, ApiMessage};
use serenity::all::{Context, Message};
use std::error::Error;

/// AI関連のWebhookメッセージを処理するハンドラ
pub async fn handle_ai_webhook(
    ctx: &Context,
    msg: &Message,
    thread_id: &str,
    res_num: &str,
    input: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("AI Webhookを受信しました。");
    println!("  スレッドID: {}", thread_id);
    println!("  レス番号: {}", res_num);
    println!("  入力テキスト: {}", input);

    // タイピングインジケータを開始
    let typing = msg.channel_id.start_typing(&ctx.http);

    // LLMに問い合わせ、結果を処理
    match crate::llm::talk_to_llama(input).await {
        Ok(response) => {
            // LLMの応答を整形
            let content_text = format!(">>{}\n{}", res_num, response);

            // APIに結果をPOST
            let api_message = ApiMessage {
                thread_id,
                cc_user_id: "AI",
                cc_user_name: "解音ゼロ",
                cc_user_avatar: 102,
                content_type: 1,
                content_text: &content_text,
                content_url: "",
            };

            if let Err(e) = unj::post_res(&api_message).await {
                eprintln!("API送信中にエラーが発生しました: {}", e);
            }
        }
        Err(e) => {
            // エラーが発生した場合はAPIには投稿せず、コンソールにエラーを出力するのみ
            eprintln!("LLMの処理中にエラーが発生しました: {}", e);
        }
    }

    // `typing`はスコープを抜けると自動で停止
    drop(typing);

    Ok(())
}
