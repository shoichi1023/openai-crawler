use std::env;

use async_openai::{
    config::AzureConfig,
    types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs, Role},
    Client,
};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let config = AzureConfig::new()
        .with_api_base(env::var("API_BASE").unwrap())
        .with_api_version(env::var("API_VERSION").unwrap())
        .with_deployment_id(env::var("DEPLOYMENT_ID").unwrap())
        .with_api_key(env::var("API_KEY").unwrap());
    let client = Client::with_config(config);

    let model = "gpt-4";
    let messages = vec![
        ChatCompletionRequestMessage {
            role: Role::Assistant,
            content: Some("今日の晩御飯はそうめんが食べたいです。".to_string()),
            name: None,
            function_call: None,
        },
        ChatCompletionRequestMessage {
            role: Role::User,
            content: Some("今日の晩御飯は？".to_string()),
            name: None,
            function_call: None,
        },
    ];
    let temperature = 0.7;
    let max_tokens: u16 = 150;
    let frequency_penalty = 0.5;
    let presence_penalty = 0.5;
    let top_p = 1.0;

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(messages)
        .temperature(temperature)
        .max_tokens(max_tokens)
        .frequency_penalty(frequency_penalty)
        .presence_penalty(presence_penalty)
        .top_p(top_p)
        .build();

    let response = client.chat().create(request.unwrap()).await.unwrap();

    println!(
        "{:#?}",
        response
            .choices
            .into_iter()
            .last()
            .unwrap()
            .message
            .content
            .unwrap_or("".to_string())
    );
}
