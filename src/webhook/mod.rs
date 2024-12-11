
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
        WebhookConfig { title, url, webhook_type, message }
    }
}


pub struct Webhook {
    pub webhook_config: WebhookConfig,
    fired: bool,
    successful: bool,
}

impl Webhook {
    pub fn new(config: WebhookConfig) -> Webhook {Webhook{ webhook_config: config, fired: false, successful: false }}
    pub fn send(&self) {
        match self.webhook_config.webhook_type {
            WebhookType::Discord => {
                let _ = discord_webhook_lib::send_discord(self.webhook_config.url.as_str(), self.webhook_config.message.as_str(), self.webhook_config.title.as_str());
            }
            WebhookType::Slack => {}
            WebhookType::Custom => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::webhook::{WebhookConfig, WebhookType};

    #[test]
    fn send_discord_webhook() {
        let url = "https://discord.com/api/webhooks/1252648442447265823/XFsq_y6hzLSxj24bbWlikcfhPJ8MGStOUjOi0vgJT83-ZLTRrcLTOqWulrIwgmlBjj6l";
        let webhook_config = WebhookConfig::new("test webhook", url, WebhookType::Discord, "hello world");
    }
}