use async_trait::async_trait;
use serde_json::json;

use crate::application::ports::EventNotifier;

/// POSTs `{event, message}` to `WEBHOOK_NOTIFY_URL` - an n8n webhook that
/// emails gilmry@gmail.com for every call it receives (workflow
/// "DernièreChance - Notifications par email"). Disabled (silent no-op) when
/// the env var isn't set. Runs detached so a slow or unreachable webhook
/// never delays the request that triggered the notification.
pub struct WebhookNotifier {
    url: Option<String>,
    client: reqwest::Client,
}

impl WebhookNotifier {
    pub fn from_env() -> Self {
        Self {
            url: std::env::var("WEBHOOK_NOTIFY_URL").ok(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl EventNotifier for WebhookNotifier {
    async fn notify(&self, event: &str, message: String) {
        let Some(url) = self.url.clone() else {
            return;
        };
        let client = self.client.clone();
        let event = event.to_string();
        tokio::spawn(async move {
            let result = client
                .post(&url)
                .json(&json!({ "event": event, "message": message }))
                .send()
                .await;
            if let Err(err) = result {
                tracing::warn!(?err, %event, "webhook notification failed");
            }
        });
    }
}
