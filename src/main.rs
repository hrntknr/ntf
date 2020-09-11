extern crate ntf;
use ntf::backends::common::Backend;
use ntf::backends::line::LineConfig;
use ntf::backends::pushover::PushoverConfig;
use ntf::backends::slack::SlackConfig;

use async_std::task;
use clap::{crate_version, App, AppSettings, Arg, SubCommand};
use config::{Config, ConfigError, File};
use std::env;
use std::vec::Vec;

fn main() {
    let config = get_config().unwrap();
    let default_title = get_title();
    let default_title = default_title.as_str();

    let app = App::new("ntf")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .version(crate_version!())
        .subcommand(
            SubCommand::with_name("send")
                .about("send notification")
                .arg(
                    Arg::with_name("title")
                        .long("title")
                        .short("t")
                        .multiple(false)
                        .help("override title")
                        .takes_value(true),
                )
                .arg(Arg::with_name("message").required(true).multiple(true)),
        );
    let matches = app.get_matches();

    if let Some(ref sub_matches) = matches.subcommand_matches("send") {
        let title = match sub_matches.value_of("title") {
            Some(title) => title,
            None => default_title,
        };
        let message = sub_matches.values_of("message").unwrap();
        let message = message.fold(String::new(), |mut acc: String, cur: &str| {
            if acc != "" {
                acc.push_str(" ");
            }
            acc.push_str(cur);
            acc
        });
        config.into_iter().for_each(|backend| {
            task::block_on(backend.send(message.as_str(), title)).unwrap();
        });
    }
}

pub fn get_title() -> String {
    let path = env::current_dir().unwrap();
    let path = path.to_str().unwrap();
    let home = dirs::home_dir().unwrap();
    let home = home.to_str().unwrap();
    let host = hostname::get().unwrap().into_string().unwrap();
    let user = username::get_user_name().unwrap();
    let relative_path = if path.starts_with(home) {
        path.replacen(home, "~", 1)
    } else {
        path.to_string()
    };
    format!(
        "{}@{}:{}",
        user.to_string(),
        host.to_string(),
        relative_path,
    )
}

fn get_config() -> Result<Vec<Box<dyn Backend>>, ConfigError> {
    let mut path = dirs::home_dir().unwrap();
    path.push(".ntf.yml");

    let mut settings = Config::default();
    settings.merge(File::from(path)).unwrap();

    let backends_str = settings.get_array("backends")?;
    let mut backends: Vec<Box<dyn Backend>> = Vec::new();

    for backend_str in backends_str {
        let settings = settings.clone();
        let backend_str = backend_str.into_str()?;
        match backend_str.as_str() {
            "line" => {
                let conf = settings.try_into::<LineConfig>()?;
                backends.push(Box::new(conf.line));
            }
            "pushover" => {
                let conf = settings.try_into::<PushoverConfig>()?;
                backends.push(Box::new(conf.pushover));
            }
            "slack" => {
                let conf = settings.try_into::<SlackConfig>()?;
                backends.push(Box::new(conf.slack));
            }
            _ => {
                return Err(ConfigError::Message(format!(
                    "invalid backend: {}",
                    backend_str.as_str()
                )));
            }
        }
    }

    Ok(backends)
}
