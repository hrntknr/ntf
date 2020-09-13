use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use syslog::Facility;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BackendError {
    #[error("`{0}`")]
    Message(String),
}

pub struct SendOption {
    pub slack_color: Option<String>,
    pub pushover_device: Option<String>,
    pub pushover_priority: Option<Priority>,
    pub pushover_retry: Option<usize>,
    pub pushover_expire: Option<usize>,
    pub syslog_facility: Option<Facility>,
    pub syslog_severity: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Priority {
    #[serde(rename = "emergency")]
    Emergency,
    #[serde(rename = "high")]
    High,
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "low")]
    Low,
    #[serde(rename = "lowest")]
    Lowest,
}

impl FromStr for Priority {
    type Err = failure::Error;

    fn from_str(s: &str) -> Result<Self, failure::Error> {
        match s {
            "emergency" => Ok(Priority::Emergency),
            "high" => Ok(Priority::High),
            "normal" => Ok(Priority::Normal),
            "low" => Ok(Priority::Low),
            "lowest" => Ok(Priority::Lowest),
            _ => Err(failure::format_err!("invalid priority: {}", s)),
        }
    }
}

#[async_trait]
pub trait Backend {
    async fn send(&self, msg: &str, title: &str, option: &SendOption) -> Result<(), BackendError>;
}
