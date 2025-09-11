use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct SdRequestBody {
    prompt: String,
    negative_prompt: String,
    steps: u32,
    sampler_name: String,
    cfg_scale: f32,
}

#[derive(Deserialize)]
struct SdApiResponse {
    images: Vec<String>,
}

const DEFAULT_NEGATIVE_PROMPT: &str = "worst quality, low quality, normal quality, blurry, out of focus, jpeg artifacts, bad anatomy, long body, bad hands, missing fingers, extra digit, mutated hands, bad face, deformed eyes, deformities, nsfw, nipples, pubic hair, text, watermark, signature, username";

pub async fn generate_image_with_sd(
    prompt: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let api_url =
        env::var("STABLE_DIFFUSION_API_URL").expect("STABLE_DIFFUSION_API_URL must be set");

    let client = Client::new();

    let body = SdRequestBody {
        prompt: prompt.to_string(),
        negative_prompt: DEFAULT_NEGATIVE_PROMPT.to_string(),
        steps: 20,
        sampler_name: "Euler a".to_string(),
        cfg_scale: 7.0,
    };

    let res = client.post(&api_url).json(&body).send().await?;

    let status = res.status();
    println!("ステータスコード: {}", status);

    if !status.is_success() {
        let error_text = res.text().await?;
        return Err(format!(
            "API request failed with status: {}. Response: {}",
            status, error_text
        )
        .into());
    }

    let json: SdApiResponse = res.json().await?;

    if let Some(image) = json.images.first() {
        Ok(image.clone())
    } else {
        Err("API response missing images".into())
    }
}
