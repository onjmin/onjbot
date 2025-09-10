use serenity::model::prelude::*;
use serenity::prelude::Context;
use std::error::Error;

// 新しいモジュールをインポート
use crate::unj::{self, ApiMessage};

pub async fn handle_beep_webhook(
    _ctx: &Context,
    _msg: &Message,
    thread_id: &str,
    _res_count: &str,
    _input: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("beep Webhookを受信しました。APIに!pongメッセージを送信します。");

    // ApiMessage構造体のインスタンスを作成
    let pong_message = ApiMessage {
        thread_id,
        cc_user_id: "AI",
        cc_user_name: "",
        cc_user_avatar: 0,
        content_type: 1,
        content_text: "!boop",
        content_url: "",
    };

    // API送信関数を呼び出す
    if let Err(e) = unj::post_res(&pong_message).await {
        eprintln!("API送信中にエラーが発生しました: {}", e);
    }

    Ok(())
}
