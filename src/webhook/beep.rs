use serenity::model::prelude::*;
use serenity::prelude::Context;
use std::error::Error;

// 新しいモジュールをインポート
use crate::unj::{self, ApiMessage};

pub async fn handle_beep_webhook(
    _ctx: &Context,
    _msg: &Message,
    thread_id: &str,
    res_num: &str,
    _input: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("beep Webhookを受信しました。APIに!pongメッセージを送信します。");

    // `res_num`変数を使ってcontent_textを動的に生成
    let content_text = format!(">>{}\n!boop", res_num);

    // ApiMessage構造体のインスタンスを作成
    let boop_message = ApiMessage {
        thread_id,
        cc_user_id: "AI",
        cc_user_name: "",
        cc_user_avatar: 0,
        content_type: 1,
        content_text: &content_text,
        content_url: "",
    };

    // API送信関数を呼び出す
    if let Err(e) = unj::post_res(&boop_message).await {
        eprintln!("API送信中にエラーが発生しました: {}", e);
    }

    Ok(())
}
