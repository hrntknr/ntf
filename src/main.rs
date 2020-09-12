extern crate ntf;
use ntf::backends::common::Backend;
use ntf::backends::line::LineConfig;
use ntf::backends::pushbullet::PushbulletConfig;
use ntf::backends::pushover::PushoverConfig;
use ntf::backends::slack::SlackConfig;

use async_std::task;
use clap::{crate_version, App, AppSettings, Arg, SubCommand};
use config::{Config, ConfigError, File};
use std::env;
use std::fs;
use std::process::{exit, Command, Stdio};
use std::time::{Duration, Instant};
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
        )
        .subcommand(
            SubCommand::with_name("done")
                .about("Execute the command and notify the message")
                .arg(
                    Arg::with_name("title")
                        .long("title")
                        .short("t")
                        .multiple(false)
                        .help("override title")
                        .takes_value(true),
                )
                .arg(Arg::with_name("cmd").required(true).multiple(true)),
        )
        .subcommand(
            SubCommand::with_name("shell-done")
                .arg(Arg::with_name("code").required(true).multiple(false))
                .arg(Arg::with_name("duration").required(true).multiple(false))
                .arg(Arg::with_name("cmd").required(true).multiple(true)),
        )
        .subcommand(SubCommand::with_name("shell-integration").about("shell-integration"));
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
        let message = unescape(message);
        config.into_iter().for_each(|backend| {
            task::block_on(backend.send(message.as_str(), title)).unwrap();
        });
    } else if let Some(ref sub_matches) = matches.subcommand_matches("done") {
        let title = match sub_matches.value_of("title") {
            Some(title) => title,
            None => default_title,
        };
        let cmd = sub_matches.values_of("cmd").unwrap();
        let cmd: Vec<String> = cmd.map(|s| s.to_string()).collect();
        let start = Instant::now();
        let cmd_exec = Command::new(&cmd[0])
            .args(&cmd[1..])
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn();
        let cmd_exec = match cmd_exec {
            Ok(ok) => ok,
            Err(err) => {
                println!("{}", err.to_string());
                exit(err.raw_os_error().unwrap())
            }
        }
        .wait();
        let code = match cmd_exec {
            Ok(ok) => ok,
            Err(err) => {
                println!("{}", err.to_string());
                exit(err.raw_os_error().unwrap())
            }
        }
        .code()
        .unwrap();
        let duration = start.elapsed();

        let message = if code == 0 {
            format!(
                "`{}` success in {}",
                cmd.join(" ").escape_default().to_string(),
                format_duration(duration),
            )
        } else {
            format!(
                "`{}` failed (code {}) in {}",
                cmd.join(" ").escape_default().to_string(),
                code,
                format_duration(duration),
            )
        };
        config.into_iter().for_each(|backend| {
            task::block_on(backend.send(message.as_str(), title)).unwrap();
        });
        exit(code);
    } else if let Some(ref sub_matches) = matches.subcommand_matches("shell-done") {
        let title = match sub_matches.value_of("title") {
            Some(title) => title,
            None => default_title,
        };

        let code = sub_matches.value_of("code").unwrap();
        let code: i32 = match code.parse() {
            Ok(code) => code,
            Err(_) => {
                println!("invalid code {}", code);
                exit(1)
            }
        };
        let duration = sub_matches.value_of("duration").unwrap();
        let duration: u64 = match duration.parse() {
            Ok(duration) => duration,
            Err(_) => {
                println!("invalid duration {}", duration);
                exit(1)
            }
        };
        let duration = Duration::from_secs(duration);
        let cmd = sub_matches.values_of("cmd").unwrap();
        let cmd: Vec<String> = cmd.map(|s| s.to_string()).collect();

        let message = if code == 0 {
            format!(
                "`{}` success in {}",
                cmd.join(" ").escape_default().to_string(),
                format_duration(duration),
            )
        } else {
            format!(
                "`{}` failed (code {}) in {}",
                cmd.join(" ").escape_default().to_string(),
                code,
                format_duration(duration),
            )
        };
        config.into_iter().for_each(|backend| {
            task::block_on(backend.send(message.as_str(), title)).unwrap();
        });
        exit(code);
    } else if let Some(ref _sub_matches) = matches.subcommand_matches("shell-integration") {
        let mut dir = dirs::data_local_dir().unwrap();
        dir.push("ntf");
        if !dir.exists() {
            match fs::create_dir_all(dir) {
                Ok(_) => (),
                Err(err) => {
                    println!("{}", err.to_string());
                    exit(err.raw_os_error().unwrap())
                }
            };
        };

        let mut file = dirs::data_local_dir().unwrap();
        file.push("ntf/ntf-shell-hook.sh");
        if !file.exists() {
            match fs::write(file, include_str!("./ntf-shell-hook.sh")) {
                Ok(_) => (),
                Err(err) => {
                    println!("{}", err.to_string());
                    exit(err.raw_os_error().unwrap())
                }
            };
        };
        {
            println!("export AUTO_NTF_DONE_LONGER_THAN=10");
            println!(
                "source {}/ntf/ntf-shell-hook.sh",
                dirs::data_local_dir().unwrap().to_str().unwrap()
            );
            println!(
                "# To use ntf's shell integration, run this and add it to your shell's rc file:"
            );
            println!("# eval \"$(ntf shell-integration)\"");
        }
    }
}

fn format_duration(duration: Duration) -> String {
    let sec = duration.as_secs();
    if sec < 60 {
        return format!("{}m {}s", sec / 60, sec % 60);
    }
    format!("{}h {}m {}s", sec / 60 / 60, (sec / 60) % 60, sec % 60)
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
            "pushbullet" => {
                let conf = settings.try_into::<PushbulletConfig>()?;
                backends.push(Box::new(conf.pushbullet));
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

fn unescape(txt: String) -> String {
    txt.replace("\\\\", "\\")
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t")
}
