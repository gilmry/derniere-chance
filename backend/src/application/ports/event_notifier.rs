use async_trait::async_trait;

/// Outbound "noteworthy event" notifications - see
/// `infrastructure::notifications::WebhookNotifier` for the adapter that
/// feeds an n8n workflow (webhook -> email). Fire-and-forget by design: a
/// notification failure must never fail the use case that triggered it.
#[async_trait]
pub trait EventNotifier: Send + Sync {
    async fn notify(&self, event: &str, message: String);
}
