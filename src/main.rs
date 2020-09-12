extern crate ntf;
use ntf::backends::common::{Backend, BackendError};
use ntf::backends::line::LineConfig;
use ntf::backends::pushbullet::PushbulletConfig;
use ntf::backends::pushover::PushoverConfig;
use ntf::backends::slack::SlackConfig;

use async_std::task;
use clap::{crate_version, App, AppSettings, Arg, ArgMatches, SubCommand};
use config::{Config, File};
use failure::{format_err, Error};
use std::env;
use std::fs;
use std::process::{exit, Command, Stdio};
use std::time::{Duration, Instant};
use std::vec::Vec;

fn main() {
    let config = match get_config() {
        Ok(config) => config,
        Err(err) => {
            println!("{}", err.to_string());
            exit(1);
        }
    };

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
        match send(config, sub_matches) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err.to_string());
                exit(1);
            }
        }
    } else if let Some(ref sub_matches) = matches.subcommand_matches("done") {
        match done(config, sub_matches) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err.to_string());
                exit(1);
            }
        }
    } else if let Some(ref sub_matches) = matches.subcommand_matches("shell-done") {
        match shell_done(config, sub_matches) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err.to_string());
                exit(1);
            }
        }
    } else if let Some(ref sub_matches) = matches.subcommand_matches("shell-integration") {
        match shell_integration(config, sub_matches) {
            Ok(_) => {}
            Err(err) => {
                println!("{}", err.to_string());
                exit(1);
            }
        }
    }
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
    let result: Result<(), BackendError> = backends
        .into_iter()
        .map(|backend| task::block_on(backend.send(message.as_str(), title.as_str())))
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
    let code = Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
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
    let result: Result<(), BackendError> = backends
        .into_iter()
        .map(|backend| task::block_on(backend.send(message.as_str(), title.as_str())))
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
    let result: Result<(), BackendError> = backends
        .into_iter()
        .map(|backend| task::block_on(backend.send(message.as_str(), title.as_str())))
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
    println!("export AUTO_NTF_DONE_LONGER_THAN=10");
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

    let mut settings = Config::default();
    settings.merge(File::from(path))?;

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
