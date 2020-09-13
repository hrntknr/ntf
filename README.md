# ntf

[![build](https://github.com/hrntknr/ntf/workflows/.github/workflows/build.yml/badge.svg)](https://github.com/hrntknr/ntf/actions?query=workflow%3A.github%2Fworkflows%2Fbuild.yml)

`ntf` brings notification to your shell. This project was inspired by [ntfy](https://github.com/dschep/ntfy).

Compared to ntfy, it has the following advantages

- Works in a single binary
- lightweight
- No need to install additional plug-ins

However, support for the backend type is poorer than ntfy.

## Quickstart

```sh
$ # for linux
$ sudo curl -L https://github.com/hrntknr/ntf/releases/download/v0.1.4/ntf-x86_64-unknown-linux-gnu -o /usr/local/bin/ntf
$ # for mac
$ # sudo curl -L https://github.com/hrntknr/ntf/releases/download/v0.1.4/ntf-x86_64-apple-darwin -o /usr/local/bin/ntf
$ sudo chmod +x /usr/local/bin/ntf

$ echo -e 'backends: ["pushover"]\npushover: {"user_key": "t0k3n"}' > ~/.ntf.yml
$ # If you want to use slack, you can do the following
$ # echo -e 'backends: ["slack"]\nslack: {"webhook: "https://hooks.slack.com/services/hogehoge"}' > ~/.ntf.yml
$
$ # send message: "test"
$ ntf send test
$ # override default setting
$ ntf send test --pushover.priority emergency --pushover.retry 60 --pushover.expire 3000
$
$ # exec command: `sleep 1` and send result
$ ntf done sleep 1
$
$ # Enable shell integration
$ echo 'AUTO_NTF_DONE_LONGER_THAN=10' >> ~/.bashrc
$ echo 'eval "$(ntf shell-integration)"' >> ~/.bashrc
```

## Supported backend

### [slack: (webhook)](https://api.slack.com/messaging/webhooks)

`~/.ntf.yml` example:

```yml
backends:
  - slack
slack:
  webhook: 'https://hooks.slack.com/services/****'
```

### [discord: (Webhook compatible with slack)](https://discord.com/developers/docs/resources/webhook)

`~/.ntf.yml` example:

```yml
backends:
  - slack
slack:
  webhook: 'https://discordapp.com/api/webhooks/****/****/slack'
  color: '#ff0000' #option
```

### [pushbullet](https://pushbullet.com/)

`~/.ntf.yml` example:

```yml
backends:
  - pushbullet
pushbullet:
  token: '********************'
```

### [pushover](https://pushover.net/)

`~/.ntf.yml` example:

```yml
backends:
  - pushover
pushover:
  user_key: '********************'
  priority: 'emergency' #option (emergency|high|normal|low|lowest)
  retry: 30 #option
  expire: 3600 #option
```

### [line](https://notify-bot.line.me/)

`~/.ntf.yml` example:

```yml
backends:
  - line
line:
  token: '********************'
```
