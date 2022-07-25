package backends

import (
	"fmt"
	"log/syslog"
)

func init() {
	backends["syslog"] = NewSyslog()
}

type SyslogConfig struct {
	Facility *string `mapstructure:"facility" validate:"omitempty,oneof=kern user mail daemon auth syslog lpr news uucp cron authpriv ftp local0 local1 local2 local3 local4 local5 local6 local7"`
	Severity *string `mapstructure:"severity" validate:"omitempty,oneof=emerg alert crit err warning notice info debug"`
}

type Syslog struct {
}

func NewSyslog() BackendInterface {
	return &Syslog{}
}

func (*Syslog) GetConfig() interface{} {
	return SyslogConfig{}
}

func (*Syslog) Send(configIface interface{}, title string, message string, status *bool) error {
	config, ok := configIface.(SyslogConfig)
	if !ok {
		return fmt.Errorf("invalid config")
	}
	var facility syslog.Priority
	if config.Facility != nil {
		switch *config.Facility {
		case "kern":
			facility = syslog.LOG_KERN
		case "user":
			facility = syslog.LOG_USER
		case "mail":
			facility = syslog.LOG_MAIL
		case "daemon":
			facility = syslog.LOG_DAEMON
		case "auth":
			facility = syslog.LOG_AUTH
		case "syslog":
			facility = syslog.LOG_SYSLOG
		case "lpr":
			facility = syslog.LOG_LPR
		case "news":
			facility = syslog.LOG_NEWS
		case "uucp":
			facility = syslog.LOG_UUCP
		case "cron":
			facility = syslog.LOG_CRON
		case "authpriv":
			facility = syslog.LOG_AUTHPRIV
		case "ftp":
			facility = syslog.LOG_FTP
		case "local0":
			facility = syslog.LOG_LOCAL0
		case "local1":
			facility = syslog.LOG_LOCAL1
		case "local2":
			facility = syslog.LOG_LOCAL2
		case "local3":
			facility = syslog.LOG_LOCAL3
		case "local4":
			facility = syslog.LOG_LOCAL4
		case "local5":
			facility = syslog.LOG_LOCAL5
		case "local6":
			facility = syslog.LOG_LOCAL6
		case "local7":
			facility = syslog.LOG_LOCAL7
		}
	} else {
		facility = syslog.LOG_USER
	}
	var severity syslog.Priority
	if config.Severity != nil {
		switch *config.Severity {
		case "emerg":
			severity = syslog.LOG_EMERG
		case "alert":
			severity = syslog.LOG_ALERT
		case "crit":
			severity = syslog.LOG_CRIT
		case "err":
			severity = syslog.LOG_ERR
		case "warning":
			severity = syslog.LOG_WARNING
		case "notice":
			severity = syslog.LOG_NOTICE
		case "info":
			severity = syslog.LOG_INFO
		case "debug":
			severity = syslog.LOG_DEBUG
		}
	} else {
		severity = syslog.LOG_INFO
	}

	logger, err := syslog.New(facility|severity, "")
	if err != nil {
		return err
	}
	defer logger.Close()
	logger.Info(message)

	return nil
}
