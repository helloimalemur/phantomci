pub mod discord;
pub mod slack;
pub mod custom_webhook;
pub mod webhook_config;

pub use crate::webhook::discord::{Webhook, WebhookConfig, WebhookType};
