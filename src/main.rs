extern crate ntf;
use ntf::backends::common::{Backend, BackendError, Priority, SendOption};
use ntf::backends::line::LineConfig;
use ntf::backends::pushbullet::PushbulletConfig;
use ntf::backends::pushover::PushoverConfig;
use ntf::backends::slack::SlackConfig;
use ntf::backends::syslog::SyslogConfig;

use async_std::task;
use clap::{Arg, ArgMatches, Command};
use config::{Config, File};
use failure::{format_err, Error};
use std::env;
use std::fs;
use std::process;
use std::time::{Duration, Instant};
use std::vec::Vec;
use syslog::Facility;

fn main() {
    let config = match get_config() {
        Ok(config) => config,
        Err(err) => {
            println!("{}", err.to_string());
            process::exit(1);
        }
    };

    let basic_args = &[
        Arg::new("title")
            .help("override title")
            .long("title")
            .short('t')
            .multiple_values(false),
        Arg::new("pushover_device")
            .help("override pushover device")
            .long("pushover.device")
            .multiple_values(false),
        Arg::new("pushover_priority")
            .help("override pushover priority")
            .long("pushover.priority")
            .multiple_values(false)
            .possible_values(&["emergency", "high", "normal", "low", "lowest"]),
        Arg::new("pushover_retry")
            .help("override pushover retry")
            .long("pushover.retry")
            .multiple_values(false),
        Arg::new("pushover_expire")
            .help("override pushover expire")
            .long("pushover.expire")
            .multiple_values(false),
        Arg::new("slack_color")
            .help("override slack color")
            .long("slack.color")
            .multiple_values(false),
        Arg::new("syslog_facility")
            .help("override syslog facility")
            .long("syslog.facility")
            .multiple_values(false)
            .possible_values(&[
                "kern", "user", "mail", "daemon", "auth", "syslog", "lpr", "news", "uucp", "cron",
                "authpriv", "ftp", "local0", "local1", "local2", "local3", "local4", "local5",
                "local6", "local7",
            ]),
        Arg::new("syslog_severity")
            .help("override syslog severity")
            .long("syslog.severity")
            .multiple_values(false)
            .possible_values(&[
                "emerg", "alert", "crit", "err", "warning", "notice", "info", "debug",
            ]),
    ];

    let app = Command::new("ntf")
        .subcommand(
            Command::new("send")
                .about("send notification")
                .args(basic_args)
                .arg(Arg::new("message").required(true).multiple_values(true)),
        )
        .subcommand(
            Command::new("done")
                .about("Execute the command and notify the message")
                .args(basic_args)
                .arg(Arg::new("cmd").required(true).multiple_values(true)),
        )
        .subcommand(
            Command::new("shell-done")
                .hide(true)
                .args(basic_args)
                .arg(Arg::new("code").required(true).multiple_values(false))
                .arg(Arg::new("duration").required(true).multiple_values(false))
                .arg(Arg::new("cmd").required(true).multiple_values(true)),
        )
        .subcommand(Command::new("shell-integration").about("shell-integration"));
    let matches = app.get_matches();

    if let Some(ref sub_matches) = matches.subcommand_matches("send") {
        match send(config, sub_matches) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err.to_string());
                process::exit(1);
            }
        }
    } else if let Some(ref sub_matches) = matches.subcommand_matches("done") {
        match done(config, sub_matches) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err.to_string());
                process::exit(1);
            }
        }
    } else if let Some(ref sub_matches) = matches.subcommand_matches("shell-done") {
        match shell_done(config, sub_matches) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err.to_string());
                process::exit(1);
            }
        }
    } else if let Some(ref sub_matches) = matches.subcommand_matches("shell-integration") {
        match shell_integration(config, sub_matches) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err.to_string());
                process::exit(1);
            }
        }
    }
}

fn get_send_option(sub_matches: &&ArgMatches) -> Result<SendOption, Error> {
    let opt = SendOption {
        slack_color: sub_matches.value_of("slack_color").map(|s| s.to_string()),
        pushover_device: sub_matches
            .value_of("pushover_device")
            .map(|s| s.to_string()),
        pushover_priority: match sub_matches
            .value_of("pushover_priority")
            .map(|s| s.parse::<Priority>())
        {
            Some(value) => Some(value?),
            None => None,
        },
        pushover_retry: match sub_matches
            .value_of("pushover_retry")
            .map(|s| s.parse::<usize>())
        {
            Some(value) => Some(value?),
            None => None,
        },
        pushover_expire: match sub_matches
            .value_of("pushover_expire")
            .map(|s| s.parse::<usize>())
        {
            Some(value) => Some(value?),
            None => None,
        },
        syslog_facility: match sub_matches
            .value_of("syslog_facility")
            .map(|s| s.parse::<Facility>())
        {
            Some(value) => Some(match value {
                Ok(facility) => facility,
                Err(_) => return Err(format_err!("can't get message")),
            }),
            None => None,
        },
        syslog_severity: sub_matches
            .value_of("syslog_severity")
            .map(|s| s.to_string()),
    };
    Ok(opt)
}

fn send(backends: Vec<Box<dyn Backend>>, sub_matches: &&ArgMatches) -> Result<(), Error> {
    let title = match sub_matches.value_of("title") {
        Some(title) => title.to_string(),
        None => get_title()?,
    };
    let message = sub_matches
        .values_of("message")
        .ok_or(format_err!("can't get message"))?;
    let message = message.fold(String::new(), |mut acc: String, cur: &str| {
        if acc != "" {
            acc.push_str(" ");
        }
        acc.push_str(cur);
        acc
    });
    let message = unescape(message);
    let opt = get_send_option(sub_matches)?;

    let result: Result<(), BackendError> = backends
        .into_iter()
        .map(|backend| task::block_on(backend.send(message.as_str(), title.as_str(), &opt)))
        .collect();
    result?;

    Ok(())
}

fn done(backends: Vec<Box<dyn Backend>>, sub_matches: &&ArgMatches) -> Result<(), Error> {
    let title = match sub_matches.value_of("title") {
        Some(title) => title.to_string(),
        None => get_title()?,
    };
    let cmd = sub_matches
        .values_of("cmd")
        .ok_or(format_err!("can't get cmd"))?;
    let cmd: Vec<String> = cmd.map(|s| s.to_string()).collect();
    let start = Instant::now();
    let code = process::Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdin(process::Stdio::inherit())
        .stdout(process::Stdio::inherit())
        .stderr(process::Stdio::inherit())
        .spawn()?
        .wait()?
        .code()
        .ok_or(format_err!("can't get code"))?;
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
    let mut opt = get_send_option(sub_matches)?;
    if opt.slack_color == None {
        match code {
            0 => opt.slack_color = Some("good".to_string()),
            _ => opt.slack_color = Some("warning".to_string()),
        }
    }

    let result: Result<(), BackendError> = backends
        .into_iter()
        .map(|backend| task::block_on(backend.send(message.as_str(), title.as_str(), &opt)))
        .collect();
    result?;

    Ok(())
}

fn shell_done(backends: Vec<Box<dyn Backend>>, sub_matches: &&ArgMatches) -> Result<(), Error> {
    let title = match sub_matches.value_of("title") {
        Some(title) => title.to_string(),
        None => get_title()?,
    };

    let code: i32 = sub_matches
        .value_of("code")
        .ok_or(format_err!("can't get code"))?
        .parse()?;
    let duration = sub_matches
        .value_of("duration")
        .ok_or(format_err!("can't get duration"))?
        .parse()?;
    let duration = Duration::from_secs(duration);
    let cmd: Vec<String> = sub_matches
        .values_of("cmd")
        .ok_or(format_err!("can't get cmd"))?
        .map(|s| s.to_string())
        .collect();

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
    let mut opt = get_send_option(sub_matches)?;
    if opt.slack_color == None {
        match code {
            0 => opt.slack_color = Some("good".to_string()),
            _ => opt.slack_color = Some("warning".to_string()),
        }
    }

    let result: Result<(), BackendError> = backends
        .into_iter()
        .map(|backend| task::block_on(backend.send(message.as_str(), title.as_str(), &opt)))
        .collect();
    result?;

    Ok(())
}

fn shell_integration(
    _backends: Vec<Box<dyn Backend>>,
    _sub_matches: &&ArgMatches,
) -> Result<(), Error> {
    let mut dir = dirs::data_local_dir().ok_or(format_err!("can't get data_local_dir"))?;
    dir.push("ntf");
    if !dir.exists() {
        fs::create_dir_all(dir)?
    };

    let mut file = dirs::data_local_dir().ok_or(format_err!("can't get data_local_dir"))?;
    file.push("ntf/ntf-shell-hook.sh");
    if !file.exists() {
        fs::write(file, include_str!("./ntf-shell-hook.sh"))?
    };
    println!("export AUTO_NTF_DONE_LONGER_THAN=${{AUTO_NTF_DONE_LONGER_THAN:=10}}");
    println!(
        "source {}/ntf/ntf-shell-hook.sh",
        dirs::data_local_dir()
            .ok_or(format_err!("can't get data_local_dir"))?
            .to_str()
            .ok_or(format_err!("can't get data_local_dir"))?
    );
    println!("# To use ntf's shell integration, run this and add it to your shell's rc file:");
    println!("# eval \"$(ntf shell-integration)\"");

    Ok(())
}

fn format_duration(duration: Duration) -> String {
    let sec = duration.as_secs();
    if sec < 60 {
        return format!("{}m {}s", sec / 60, sec % 60);
    }
    format!("{}h {}m {}s", sec / 60 / 60, (sec / 60) % 60, sec % 60)
}

pub fn get_title() -> Result<String, Error> {
    let path = env::current_dir()?;
    let path = path.to_str().ok_or(format_err!("can't get current_dir"))?;
    let home = dirs::home_dir().ok_or(format_err!("can't get home_dir"))?;
    let home = home.to_str().ok_or(format_err!("can't get home_dir"))?;
    let host = hostname::get()?.into_string().unwrap();
    let user = username::get_user_name()?;
    let relative_path = if path.starts_with(home) {
        path.replacen(home, "~", 1)
    } else {
        path.to_string()
    };

    Ok(format!(
        "{}@{}:{}",
        user.to_string(),
        host.to_string(),
        relative_path,
    ))
}

fn get_config() -> Result<Vec<Box<dyn Backend>>, Error> {
    let mut path = dirs::home_dir().ok_or(format_err!("can't get home_dir"))?;
    path.push(".ntf.yml");

    let settings = Config::builder().add_source(File::from(path)).build()?;

    let backends_str = settings.get_array("backends")?;
    let mut backends: Vec<Box<dyn Backend>> = Vec::new();

    for backend_str in backends_str {
        let settings = settings.clone();
        let backend_str = backend_str.into_string()?;
        match backend_str.as_str() {
            "line" => {
                let conf: LineConfig = settings.try_deserialize()?;
                backends.push(Box::new(conf.line));
            }
            "pushbullet" => {
                let conf: PushbulletConfig = settings.try_deserialize()?;
                backends.push(Box::new(conf.pushbullet));
            }
            "pushover" => {
                let conf: PushoverConfig = settings.try_deserialize()?;
                backends.push(Box::new(conf.pushover));
            }
            "slack" => {
                let conf: SlackConfig = settings.try_deserialize()?;
                backends.push(Box::new(conf.slack));
            }
            "syslog" => {
                let conf: SyslogConfig = settings.try_deserialize()?;
                backends.push(Box::new(conf.syslog));
            }
            _ => {
                return Err(format_err!("invalid backend: {}", backend_str.as_str()));
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

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn get_title_test() {
        let title = get_title().unwrap();
        let re = Regex::new(r"^.+@.+:.+$").unwrap();
        assert!(re.is_match(title.as_str()));
    }

    #[test]
    fn unescape_test() {
        let test = "\\n\\r\\t\\\\\\'\\\"\\0";
        let expect = "\n\r\t\\\\'\\\"\\0";
        assert_eq!(unescape(test.to_string()), expect);
    }
}
