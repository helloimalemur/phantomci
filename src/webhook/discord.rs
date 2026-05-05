use discord_webhook_lib::DiscordMessage;
use crate::webhook::custom_webhook::send_custom;
use crate::webhook::slack::send_slack;
use serde_json::json;
use tokio::time::{sleep, Duration};

pub enum WebhookType {
    Discord,
    Slack,
    Custom,
}

pub struct WebhookConfig {
    title: String,
    url: String,
    pub webhook_type: WebhookType,
    message: String,
}

impl WebhookConfig {
    pub fn new(title: &str, url: &str, webhook_type: WebhookType, message: &str) -> WebhookConfig {
        let title = title.to_string();
        let url = url.to_string();
        let message = message.to_string();
        WebhookConfig {
            title,
            url,
            webhook_type,
            message,
        }
    }
}

pub struct Webhook {
    pub webhook_config: WebhookConfig,
    // fired: bool,
    // successful: bool,
}

impl Webhook {
    pub fn new(config: WebhookConfig) -> Webhook {
        Webhook {
            webhook_config: config,
            // fired: false,
            // successful: false,
        }
    }
    pub async fn send(&self) {
        match self.webhook_config.webhook_type {
            WebhookType::Discord => {
                let mut chunks = chunk_message(&self.webhook_config.message, 2000);
                if chunks.is_empty() {
                    chunks.push("".to_string());
                }
                let total_chunks = chunks.len();

                for (i, chunk) in chunks.iter().enumerate() {
                    let mut message = DiscordMessage::builder(self.webhook_config.url.as_str());
                    message.add_message(chunk.as_str());

                    if i == 0 {
                        message.add_field("title", self.webhook_config.title.as_str());
                    }

                    let sender = message.build();

                    if let Err(e) = sender.send().await {
                        eprintln!("Discord webhook error: {}", e);
                    }

                    if i < total_chunks - 1 {
                        sleep(Duration::from_millis(1000)).await;
                    }
                }
            }
            WebhookType::Slack => {
                send_slack(
                    self.webhook_config.url.as_str(),
                    self.webhook_config.message.as_str(),
                    Some(self.webhook_config.title.as_str()),
                )
                .await;
            }
            WebhookType::Custom => {
                let body = json!({
                    "title": self.webhook_config.title.as_str(),
                    "message": self.webhook_config.message.as_str(),
                })
                .to_string();
                send_custom(
                    self.webhook_config.url.as_str(),
                    body.as_str(),
                    "application/json",
                )
                .await;
            }
        }
    }
}

fn chunk_message(message: &str, chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut chars = message.chars();
    loop {
        let chunk: String = chars.by_ref().take(chunk_size).collect();
        if chunk.is_empty() {
            break;
        }
        chunks.push(chunk);
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_message() {
        let message = "abcdefghij";
        let chunks = chunk_message(message, 3);
        assert_eq!(chunks, vec!["abc", "def", "ghi", "j"]);
    }

    #[test]
    fn test_chunk_message_utf8() {
        let message = "🦀🦀🦀🦀🦀";
        let chunks = chunk_message(message, 2);
        assert_eq!(chunks, vec!["🦀🦀", "🦀🦀", "🦀"]);
    }

    #[test]
    fn test_chunk_message_empty() {
        let message = "";
        let chunks = chunk_message(message, 2000);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_chunk_message_exact() {
        let message = "abc";
        let chunks = chunk_message(message, 3);
        assert_eq!(chunks, vec!["abc"]);
    }
}
