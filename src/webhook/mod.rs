use discord_webhook_lib::send_discord;

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
                if let Err(e) = send_discord(
                    self.webhook_config.url.as_str(),
                    self.webhook_config.message.as_str(),
                    self.webhook_config.title.as_str(),
                )
                .await
                {
                    eprintln!("{}", e)
                }
            }
            WebhookType::Slack => {}
            WebhookType::Custom => {}
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
