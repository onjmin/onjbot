use crate::llm::talk_to_llama;
use serenity::all::{CommandInteraction, Context};
use serenity::all::{CreateInteractionResponse, CreateInteractionResponseMessage};

pub async fn handle_chat_command(ctx: &Context, command: &CommandInteraction, user_input: &str) {
    match talk_to_llama(user_input).await {
        Ok(response) => {
            let _ = command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        serenity::builder::CreateInteractionResponseMessage::default()
                            .content(response),
                    ),
                )
                .await;
        }
        Err(e) => {
            let _ = command
                .create_response(
                    &ctx.http,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::default()
                            .content(format!("エラー: {}", e)),
                    ),
                )
                .await;
        }
    }
}
