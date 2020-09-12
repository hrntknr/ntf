use super::common::{Backend, BackendError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://api.pushbullet.com/v2/pushes";

#[derive(Deserialize, Serialize)]
struct Body {
    #[serde(rename = "type")]
    _type: String,
    title: String,
    body: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PushbulletConfig {
    pub pushbullet: PushbulletBackend,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PushbulletBackend {
    token: String,
}

#[async_trait]
impl Backend for PushbulletBackend {
    async fn send(&self, msg: &str, title: &str) -> Result<(), BackendError> {
        let body = &Body {
            _type: "note".to_string(),
            title: title.to_string(),
            body: msg.to_string(),
        };
        let req = match surf::post(API_URL)
            .set_header("Access-Token", self.token.to_string())
            .set_header("Content-Type", "application/json")
            .body_form(body)
        {
            Ok(req) => req,
            Err(err) => return Err(BackendError::Message(err.to_string())),
        };
        match req.await {
            Ok(res) => (),
            Err(err) => return Err(BackendError::Message(err.to_string())),
        };

        Ok(())
    }
}
