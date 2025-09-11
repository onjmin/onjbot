// src/post_unj_res.rs

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use image::ImageFormat;
use reqwest::Client;
use std::env;
use std::error::Error as StdError;
use std::error::Error;
use std::io::Cursor;

use std::fs::{self, OpenOptions};
use std::io::Write;

/// base64 画像を受け取り、再エンコード (EXIF削除) してアップロードし、内部キーを返す
pub async fn upload_image_base64(
    base64_str: &str,
) -> Result<String, Box<dyn StdError + Send + Sync>> {
    let base_url = env::var("FEEDER_ROOM_URL").expect("FEEDER_ROOM_URL must be set");
    let cookie = env::var("FEEDER_COOKIE").expect("FEEDER_COOKIE must be set");
    let upload_url = format!("{}post_media_xhr.php", base_url);

    // --- base64 文字列からデコード ---
    let bytes = BASE64_STANDARD.decode(base64_str)?;

    // --- image crate で読み込み & PNG に再エンコード (EXIF除去) ---
    let img = image::load_from_memory(&bytes)?;
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::Png)?;
    let png_bytes = buf.into_inner();

    // --- multipart/form-data で送信 ---
    let client = Client::new();
    let form = reqwest::multipart::Form::new().part(
        "media",
        reqwest::multipart::Part::bytes(png_bytes)
            .file_name("upload.png")
            .mime_str("image/png")?,
    );

    let resp = client
        .post(&upload_url)
        .header("Cookie", cookie)
        .multipart(form)
        .send()
        .await?;
    let text = resp.text().await?; // 例: "7,897a1633cc0a93f4"

    // --- image_id だけ抽出 ---
    if let Some(pos) = text.find(',') {
        Ok(text[..pos].to_string()) // カンマの前が image_id
    } else {
        Err(format!("予期しないレスポンス形式: {}", text).into())
    }
}

/// 内部キーから実際の画像URLを取得する (認証必須の場合あり)
pub async fn get_image_url(picture_id: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
    let base_url = env::var("FEEDER_ROOM_URL").expect("FEEDER_ROOM_URL must be set");
    let cookie = env::var("FEEDER_COOKIE").expect("FEEDER_COOKIE must be set");
    let client = Client::new();
    let url = format!("{}settings/manage_pictures.php", base_url);

    let html = client
        .get(&url)
        .header("Cookie", cookie)
        .send()
        .await?
        .text()
        .await?;

    // `<input id="15"` を探す
    if let Some(start) = html.find(&format!(r#"<input id="{}""#, picture_id)) {
        // そこから行末まで切り出す
        let slice = &html[start..];

        // href="../pictures/XXXX.png" を探す
        if let Some(href_pos) = slice.find("href=\"../pictures/") {
            let rest = &slice[href_pos + 6..]; // `href="` の後ろから
            if let Some(end_quote) = rest.find('"') {
                let rel_path = &rest[..end_quote]; // ../pictures/PIC_xxx.png
                let clean_path = rel_path.trim_start_matches("../");
                let abs_url = format!("{}{}", base_url, clean_path);
                return Ok(abs_url);
            }
        }
    }

    // 見つからなかった場合にログにレスポンスを吐く
    let log_msg = format!(
        "画像URL取得に失敗しました（認証エラーの可能性あり）: picture_id {}\nHTMLレスポンス:\n{}\n\n",
        picture_id, html
    );

    fs::create_dir_all("logs")?; // 存在していなければ作る
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/feeder_error.log")
    {
        let _ = file.write_all(log_msg.as_bytes());
    }

    Err(format!("picture_id {} の画像が見つかりません", picture_id).into())
}
