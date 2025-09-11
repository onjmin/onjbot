use crate::unj::{self, ApiMessage};
use serenity::all::{Context, Message};
use std::error::Error;

/// 画像生成関連のWebhookメッセージを処理するハンドラ
pub async fn handle_image_gen_webhook(
    _ctx: &Context,
    _msg: &Message,
    thread_id: &str,
    res_num: &str,
    input: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("画像生成 Webhookを受信しました。");
    println!("  スレッドID: {}", thread_id);
    println!("  レス番号: {}", res_num);
    println!("  入力テキスト: {}", input);

    // Stable Diffusionで画像生成
    match crate::stable_diffusion::generate_image_with_sd(input).await {
        Ok(base64_img) => {
            let clean_base64 = base64_img
                .strip_prefix("data:image/png;base64,")
                .unwrap_or(&base64_img);

            // 画像アップロード
            match crate::feeder::upload_image_base64(clean_base64).await {
                Ok(image_id) => {
                    // 画像URL取得
                    match crate::feeder::get_image_url(&image_id).await {
                        Ok(image_url) => {
                            // UNJに投稿
                            let api_message = ApiMessage {
                                thread_id,
                                cc_user_id: "AI",
                                cc_user_name: "★AI",
                                cc_user_avatar: 0,
                                content_type: 4,
                                content_text: &format!(">>{}", res_num),
                                content_url: &image_url,
                            };

                            if let Err(e) = unj::post_res(&api_message).await {
                                eprintln!("API送信中にエラーが発生しました: {}", e);
                            }
                        }
                        Err(e) => {
                            eprintln!("画像URL取得に失敗しました（認証エラーの可能性あり）: {}", e);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("画像アップロードに失敗しました: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("画像生成に失敗しました: {}", e);
        }
    }

    Ok(())
}
