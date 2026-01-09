pub mod discord;
pub mod slack;
pub mod custom_webhook;
pub mod webhook_config;

pub use crate::webhook::discord::{Webhook, WebhookConfig, WebhookType};
pub use crate::webhook::slack::{send_slack};
pub use crate::webhook::custom_webhook::{send_custom};