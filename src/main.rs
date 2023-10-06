use std::env;

use async_openai::{
    config::AzureConfig,
    types::{
        ChatCompletionFunctions, ChatCompletionRequestMessage, CreateChatCompletionRequestArgs,
        Role,
    },
    Client,
};
use async_recursion::async_recursion;
use serde_json::json;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let input =
        ChatCompletionRequestMessage {
            role: Role::User,
            content: Some("function callingについて下記サイトを元に教えてください。 https://dev.classmethod.jp/articles/understand-openai-function-calling/".to_string()),
            name: None,
            function_call: None,
        };

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

    let message = recursive_function_calling(functions, vec![input])
        .await
        .unwrap();

    println!("{:#?}", message);
}

#[async_recursion]
async fn recursive_function_calling(
    functions: Vec<ChatCompletionFunctions>,
    messages: Vec<ChatCompletionRequestMessage>,
) -> anyhow::Result<String> {
    let config = AzureConfig::new()
        .with_api_base(env::var("API_BASE").unwrap())
        .with_api_version(env::var("API_VERSION").unwrap())
        .with_deployment_id(env::var("DEPLOYMENT_ID").unwrap())
        .with_api_key(env::var("API_KEY").unwrap());
    let client = Client::with_config(config);

    let model = "gpt-4-0613";
    let temperature = 0.7;
    let max_tokens: u16 = 1600;
    let frequency_penalty = 0.0;
    let presence_penalty = 0.0;
    let top_p = 0.95;

    let request = CreateChatCompletionRequestArgs::default()
        .model(model)
        .messages(messages.clone())
        .functions(functions.clone())
        .temperature(temperature)
        .max_tokens(max_tokens)
        .frequency_penalty(frequency_penalty)
        .presence_penalty(presence_penalty)
        .top_p(top_p)
        .build();

    let response = client.chat().create(request.unwrap()).await.unwrap();
    let message = response.choices.first().unwrap().message.clone();
    let mut text = message.content.clone().unwrap_or("".to_string());

    if message.clone().function_call.is_some() {
        println!(
            "function calling: {:#?}",
            message.clone().function_call.unwrap().name
        );
        let result = match &*message.clone().function_call.unwrap().name {
            "get_html_context" => {
                let url = serde_json::from_str::<serde_json::Value>(
                    &message.clone().function_call.unwrap().arguments,
                )?
                .get("url")
                .unwrap()
                .to_string()
                .replace("\"", "");
                println!("url: {}", url);
                get_html_context(&url).await.unwrap()
            }
            _ => "".to_string(),
        };

        text = recursive_function_calling(
            functions,
            vec![
                vec![ChatCompletionRequestMessage {
                    role: Role::Assistant,
                    content: Some(result),
                    name: None,
                    function_call: None,
                }],
                messages,
            ]
            .concat(),
        )
        .await?;
    }

    Ok(text)
}

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
    let result: Vec<&str> = text.split("\n").collect();
    Ok((&result[30..result.len() - 150]).join("\n").to_string())
}

#[cfg(test)]
mod test {
    use crate::get_html_context;

    #[tokio::test]
    async fn test_get_html_context() {
        let url = "https://dev.classmethod.jp/articles/understand-openai-function-calling/";
        let text = get_html_context(url).await.unwrap();
        println!("{}", text);
    }
}
