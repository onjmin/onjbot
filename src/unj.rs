// src/post_unj_res.rs

use reqwest::Client;
use serde::Serialize;
use std::env;
use std::error::Error as StdError;

// APIエンドポイントに送信するJSONボディの構造体
#[derive(Serialize)]
pub struct ApiMessage<'a> {
    #[serde(rename = "threadId")]
    pub thread_id: &'a str,
    #[serde(rename = "ccUserId")]
    pub cc_user_id: &'a str,
    #[serde(rename = "ccUserName")]
    pub cc_user_name: &'a str,
    #[serde(rename = "ccUserAvatar")]
    pub cc_user_avatar: u8,
    #[serde(rename = "contentType")]
    pub content_type: u8,
    #[serde(rename = "contentText")]
    pub content_text: &'a str,
    #[serde(rename = "contentUrl")]
    pub content_url: &'a str,
}

// 修正後: 構造体の参照を引数として受け取るように変更
pub async fn post_res(body: &ApiMessage<'_>) -> Result<(), Box<dyn StdError + Send + Sync>> {
    let api_endpoint =
        env::var("UNJ_ADMIN_API_ENDPOINT").expect("UNJ_ADMIN_API_ENDPOINT must be set");
    let api_key = env::var("UNJ_ADMIN_API_KEY").expect("UNJ_ADMIN_API_KEY must be set");

    let client = Client::new();
    let url = format!("{}api/admin/thread/res", api_endpoint);

    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", &api_key)
        .json(body) // `body`を直接渡す
        .send()
        .await;

    match response {
        Ok(res) => {
            if res.status().is_success() {
                println!("APIへのメッセージ送信に成功しました。");
                Ok(())
            } else {
                let status = res.status();
                let text = res.text().await.unwrap_or_default();
                Err(format!(
                    "APIへのメッセージ送信に失敗しました。ステータスコード: {}, レスポンス: {}",
                    status, text
                )
                .into())
            }
        }
        Err(e) => Err(e.into()),
    }
}
