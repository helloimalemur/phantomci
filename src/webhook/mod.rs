
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
    pub fn new(title: String, url: String, webhook_type: WebhookType, message: String) -> WebhookConfig {
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