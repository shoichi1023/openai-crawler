use std::env;

use async_openai::{
    config::AzureConfig,
    types::{
        ChatCompletionFunctions, ChatCompletionRequestMessage, CreateChatCompletionRequestArgs,
        Role,
    },
    Client,
};
use serde_json::json;

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
            role: Role::User,
            content: Some("function callingについて下記サイトを元に教えてください。 https://dev.classmethod.jp/articles/understand-openai-function-calling/".to_string()),
            name: None,
            function_call: None,
        },
    ];

    let functions = vec![ChatCompletionFunctions {
        name: "get_html_context".to_string(),
        description: Some("webサイトのurlを元にそのページの内容を取得する".to_string()),
        parameters: Some(json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "webサイトのurl 例: https://www.google.com, https://note.com/pharmax/n/n56fbcfa51e48",
                },
            },
            "required": ["cities"],
        })),
    }];
    let temperature = 0.7;
    let max_tokens: u16 = 1600;
    let frequency_penalty = 0.0;
    let presence_penalty = 0.0;
    let top_p = 0.95;

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(messages)
        .functions(functions)
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
// async fn main() {
//     let url = "https://dev.classmethod.jp/articles/understand-openai-function-calling/";
//     let html = get_html_context(url).await.unwrap();
//     println!("{}", html);
// }

async fn get_html_context(url: &str) -> anyhow::Result<String> {
    let user_agent_value: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/109.0.0.0 Safari/537.36";
    let client = reqwest::Client::new();
    let res = client
        .get(url)
        .header("User-Agent", user_agent_value)
        .send()
        .await?
        .text()
        .await?;
    let text = html2text::from_read(res.as_bytes(), 100);
    Ok(text)
}
