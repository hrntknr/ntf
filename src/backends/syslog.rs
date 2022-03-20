use super::common::{Backend, BackendError, SendOption};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process;
use syslog::{Facility, Formatter3164};

#[derive(Serialize, Deserialize, Debug)]
pub struct SyslogConfig {
    pub syslog: SyslogBackend,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SyslogBackend {
    facility: Option<String>,
    severity: Option<String>,
}

#[async_trait]
impl Backend for SyslogBackend {
    async fn send(&self, msg: &str, _title: &str, option: &SendOption) -> Result<(), BackendError> {
        let formatter = Formatter3164 {
            facility: match option.syslog_facility {
                Some(facility) => facility,
                None => match self.facility.clone() {
                    Some(facility) => match facility.parse() {
                        Ok(facility) => facility,
                        Err(_) => {
                            return Err(BackendError::Message(
                                "failed to parse facility".to_string(),
                            ))
                        }
                    },
                    None => Facility::LOG_USER,
                },
            },
            hostname: None,
            process: "ntf".into(),
            pid: process::id() as u32,
        };
        let mut writer = match syslog::unix(formatter) {
            Ok(writer) => writer,
            Err(_) => {
                return Err(BackendError::Message(
                    "faild to get syslog writer".to_string(),
                ))
            }
        };

        let result = match option.syslog_severity.clone() {
            Some(severity) => match severity.as_str() {
                "emerg" => writer.emerg(msg),
                "alert" => writer.alert(msg),
                "crit" => writer.crit(msg),
                "err" => writer.err(msg),
                "warning" => writer.warning(msg),
                "notice" => writer.notice(msg),
                "info" => writer.info(msg),
                "debug" => writer.debug(msg),
                _ => {
                    return Err(BackendError::Message(format!(
                        "{} is not valid severity",
                        severity
                    )))
                }
            },
            None => match self.severity.clone() {
                Some(severity) => match severity.as_str() {
                    "emerg" => writer.emerg(msg),
                    "alert" => writer.alert(msg),
                    "crit" => writer.crit(msg),
                    "err" => writer.err(msg),
                    "warning" => writer.warning(msg),
                    "notice" => writer.notice(msg),
                    "info" => writer.info(msg),
                    "debug" => writer.debug(msg),
                    _ => {
                        return Err(BackendError::Message(format!(
                            "{} is not valid severity",
                            severity
                        )))
                    }
                },
                None => writer.info(msg),
            },
        };

        match result {
            Ok(_) => (),
            Err(err) => {
                return Err(BackendError::Message(format!(
                    "faild to send syslog: {}",
                    err.to_string()
                )))
            }
        }

        Ok(())
    }
}
