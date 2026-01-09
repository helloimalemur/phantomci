use discord_webhook_lib::DiscordMessage;
use crate::webhook::custom_webhook::send_custom;
use crate::webhook::slack::send_slack;
use serde_json::json;

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
                let mut message = DiscordMessage::builder(self.webhook_config.url.as_str());
                message.add_message(self.webhook_config.message.as_str());
                message.add_field("title", self.webhook_config.title.as_str());
                let sender = message.build();

                if let Err(e) = sender.send().await {
                    eprintln!("{}", e)
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

// #[cfg(test)]
// mod tests {
//     use crate::webhook::{Webhook, WebhookConfig, WebhookType};
//     use std::env;
//
//     #[test]
//     async fn send_discord_webhook() {
//         if let Ok(wh_url) = env::var("DISCORD_WEBHOOK_URL") {
//             let webhook = Webhook::new(WebhookConfig::new(
//                 "test webhook",
//                 wh_url.as_str(),
//                 WebhookType::Discord,
//                 "hello world",
//             ));
//             webhook.send().await;
//         }
//     }
// }
