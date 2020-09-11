# ntf

`ntf` brings notification to your shell. This project was inspired by [ntfy](https://github.com/dschep/ntfy).

Compared to ntfy, it has the following advantages

- Works in a single binary
- lightweight
- No need to install additional plug-ins

However, support for the backend type is poorer than ntfy.

## Quickstart

```sh
$ sudo curl -L https://github.com/hrntknr/ntf/releases/download/v0.1.0/ntf-linux-amd64 -o /usr/local/bin/ntf
$ sudo chmod +x /usr/local/bin/ntf

$ echo -e 'backends: ["pushover"]\npushover: {"user_key": "t0k3n"}' > ~/.ntf.yml
$ # If you want to use slack, you can do the following
$ # echo -e 'backends: ["slack"]\nslack: {"webhook: "https://hooks.slack.com/services/hogehoge"}' > ~/.ntf.yml
$ ntf send test
```

## Supported backend

- slack(incoming webhook)
- pushover
