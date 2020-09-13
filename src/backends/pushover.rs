use super::common::{Backend, BackendError, Priority, SendOption};
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
    device: Option<String>,
    priority: Option<isize>,
    retry: Option<usize>,
    expire: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PushoverConfig {
    pub pushover: PushoverBackend,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PushoverBackend {
    user_key: String,
    device: Option<String>,
    priority: Option<Priority>,
    retry: Option<usize>,
    expire: Option<usize>,
}

#[async_trait]
impl Backend for PushoverBackend {
    async fn send(&self, msg: &str, title: &str, option: &SendOption) -> Result<(), BackendError> {
        let body = &Body {
            token: API_TOKEN.to_string(),
            user: self.user_key.to_string(),
            title: title.to_string(),
            message: msg.to_string(),
            device: match option.pushover_device.clone() {
                Some(device) => Some(device),
                None => match self.device.clone() {
                    Some(device) => Some(device),
                    None => None,
                },
            },
            priority: match &option.pushover_priority {
                Some(priority) => Some(priority_to_pushover_code(priority)),
                None => match &self.priority {
                    Some(priority) => Some(priority_to_pushover_code(priority)),
                    None => None,
                },
            },
            retry: match option.pushover_retry {
                Some(retry) => Some(retry),
                None => match self.retry {
                    Some(retry) => Some(retry),
                    None => None,
                },
            },
            expire: match option.pushover_expire {
                Some(expire) => Some(expire),
                None => match self.expire {
                    Some(expire) => Some(expire),
                    None => None,
                },
            },
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

pub fn priority_to_pushover_code(priority: &Priority) -> isize {
    match priority {
        Priority::Emergency => 2,
        Priority::High => 1,
        Priority::Normal => 0,
        Priority::Low => -1,
        Priority::Lowest => -2,
    }
}
