use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct RequestMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct RequestBody {
    messages: Vec<RequestMessage>,
    mode: String,
    character: String,
    temperature: f32,
    top_p: f32,
    top_k: u32,
}

#[derive(Deserialize)]
struct ApiResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

pub async fn talk_to_llama(
    user_prompt: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let api_url = env::var("LLAMA_API_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:5000/v1/chat/completions".to_string());

    let client = Client::new();

    let body = RequestBody {
        messages: vec![RequestMessage {
            role: "user".to_string(),
            content: user_prompt.to_string(),
        }],
        mode: "chat-instruct".to_string(),
        character: "解音ゼロv2".to_string(),
        temperature: 0.6,
        top_p: 0.95,
        top_k: 20,
    };

    let res = client.post(&api_url).json(&body).send().await?;

    let status = res.status();
    println!("ステータスコード: {}", status);

    if !status.is_success() {
        return Err(format!("API request failed with status: {}", status).into());
    }

    let json: ApiResponse = res.json().await?;

    if let Some(choice) = json.choices.first() {
        Ok(choice.message.content.clone())
    } else {
        Err("API response missing choices".into())
    }
}
