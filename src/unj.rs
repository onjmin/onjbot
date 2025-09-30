// src/post_unj_res.rs

use reqwest::Client;
use serde::{Deserialize, Serialize};
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
    let api_url = env::var("UNJ_ADMIN_API_URL").expect("UNJ_ADMIN_API_URL must be set");
    let api_key = env::var("UNJ_ADMIN_API_KEY").expect("UNJ_ADMIN_API_KEY must be set");

    let client = Client::new();
    let url = format!("{}api/admin/thread/res", api_url);

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

// GET /emergency/deny-all のレスポンス
#[derive(Debug, Deserialize)]
pub struct DenyAllResponse {
    #[serde(rename = "denyAll")]
    pub deny_all: bool,
}

// POST /emergency/deny-all のリクエストボディ
#[derive(Debug, Serialize)]
pub struct DenyAllRequest {
    #[serde(rename = "denyAll")]
    pub deny_all: bool,
}

// GET: 現在の denyAll フラグを取得
pub async fn get_deny_all() -> Result<DenyAllResponse, Box<dyn StdError + Send + Sync>> {
    let api_url = env::var("UNJ_ADMIN_API_URL").expect("UNJ_ADMIN_API_URL must be set");
    let api_key = env::var("UNJ_ADMIN_API_KEY").expect("UNJ_ADMIN_API_KEY must be set");

    let client = Client::new();
    let url = format!("{}api/admin/emergency/deny-all", api_url);

    let res = client
        .get(&url)
        .header("Authorization", &api_key)
        .send()
        .await?;

    if res.status().is_success() {
        let body = res.json::<DenyAllResponse>().await?;
        Ok(body)
    } else {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        Err(format!("denyAll GET 失敗: status={} body={}", status, text).into())
    }
}

// POST: denyAll フラグを更新 (true/false)
pub async fn post_deny_all(flag: bool) -> Result<DenyAllResponse, Box<dyn StdError + Send + Sync>> {
    let api_url = env::var("UNJ_ADMIN_API_URL").expect("UNJ_ADMIN_API_URL must be set");
    let api_key = env::var("UNJ_ADMIN_API_KEY").expect("UNJ_ADMIN_API_KEY must be set");

    let client = Client::new();
    let url = format!("{}api/admin/emergency/deny-all", api_url);

    let body = DenyAllRequest { deny_all: flag };

    let res = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Authorization", &api_key)
        .json(&body)
        .send()
        .await?;

    if res.status().is_success() {
        let body = res.json::<DenyAllResponse>().await?;
        Ok(body)
    } else {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        Err(format!("denyAll POST 失敗: status={} body={}", status, text).into())
    }
}
