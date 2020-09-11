use super::common::{Backend, BackendError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://notify-api.line.me/api/notify";

#[derive(Deserialize, Serialize)]
struct Body {
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LineConfig {
    pub line: LineBackend,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LineBackend {
    token: String,
}

#[async_trait]
impl Backend for LineBackend {
    async fn send(&self, msg: &str, title: &str) -> Result<(), BackendError> {
        let body = &Body {
            message: format!("{}\n{}", title, msg.to_string()),
        };
        let req = match surf::post(API_URL)
            .set_header("Authorization", format!("Bearer {}", self.token))
            .body_form(body)
        {
            Ok(req) => req,
            Err(err) => return Err(BackendError::Message(err.to_string())),
        };
        match req.await {
            Ok(_) => (),
            Err(err) => return Err(BackendError::Message(err.to_string())),
        };

        Ok(())
    }
}
