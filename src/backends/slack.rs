use super::common::{Backend, BackendError, SendOption};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const COLOR: &str = "good";
const ICON_URL: &str = "https://raw.githubusercontent.com/hrntknr/ntf/master/assets/icon.png";

#[derive(Deserialize, Serialize)]
struct Body {
    icon_url: String,
    attachments: Vec<Attachment>,
}

#[derive(Deserialize, Serialize)]
struct Attachment {
    title: String,
    text: String,
    color: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SlackConfig {
    pub slack: SlackBackend,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SlackBackend {
    webhook: String,
    color: Option<String>,
}

#[async_trait]
impl Backend for SlackBackend {
    async fn send(&self, msg: &str, title: &str, option: &SendOption) -> Result<(), BackendError> {
        let mut attachments = Vec::new();
        attachments.push(Attachment {
            title: title.to_string(),
            text: msg.to_string(),
            color: match option.slack_color.clone() {
                Some(color) => color,
                None => match self.color.clone() {
                    Some(color) => color,
                    None => COLOR.to_string(),
                },
            },
        });
        let body = &Body {
            icon_url: ICON_URL.to_string(),
            attachments: attachments,
        };
        let req = match surf::post(&self.webhook).body_json(body) {
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
