use crate::networking::Endpoint;

#[async_trait::async_trait]
pub trait Network: Send + Sync {
    async fn endpoint(&self) -> Endpoint;
}
