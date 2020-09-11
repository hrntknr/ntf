use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BackendError {
    #[error("`{0}`")]
    Message(String),
}

#[async_trait]
pub trait Backend {
    async fn send(&self, msg: &str, title: &str) -> Result<(), BackendError>;
}
