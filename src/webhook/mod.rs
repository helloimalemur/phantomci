
enum WebhookType {
    Discord,
    Slack,
    Custom,
}

struct WebhookConfig {
    url: String,
    webhook_type: WebhookType
}

impl WebhookConfig {
    pub fn new(url: String, webhook_type: WebhookType) -> WebhookConfig {
        WebhookConfig { url, webhook_type }
    }
}


struct Webhook {
    webhook_config: WebhookConfig
}