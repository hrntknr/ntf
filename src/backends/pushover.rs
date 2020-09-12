use super::common::{Backend, BackendError};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const API_TOKEN: &str = "abughxjjtuofgt89bz21mibut67j5t";
const API_URL: &str = "https://api.pushover.net/1/messages.json";

#[derive(Deserialize, Serialize)]
struct Body {
    token: String,
    user: String,
    title: String,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PushoverConfig {
    pub pushover: PushoverBackend,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PushoverBackend {
    user_key: String,
}

#[async_trait]
impl Backend for PushoverBackend {
    async fn send(&self, msg: &str, title: &str) -> Result<(), BackendError> {
        let body = &Body {
            token: API_TOKEN.to_string(),
            user: self.user_key.to_string(),
            title: title.to_string(),
            message: msg.to_string(),
        };
        let req = match surf::post(API_URL).body_json(body) {
            Ok(req) => req,
            Err(err) => return Err(BackendError::Message(err.to_string())),
        };
        let res = match req.await {
            Ok(res) => res,
            Err(err) => return Err(BackendError::Message(err.to_string())),
        };
        if res.status() != 200 {
            return Err(BackendError::Message(res.status().to_string()));
        }

        Ok(())
    }
}
