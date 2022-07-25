package backends

import (
	"fmt"
	"net/http"
	"net/url"
	"strings"
)

func init() {
	backends["line"] = NewLine()
}

const LINE_API_URL = "https://notify-api.line.me/api/notify"

type LineConfig struct {
	Token string `mapstructure:"token" validate:"required"`
}

type Line struct {
}

func NewLine() BackendInterface {
	return &Line{}
}

func (*Line) GetConfig() interface{} {
	return LineConfig{}
}

func (*Line) Send(configIface interface{}, title string, message string, status *bool) error {
	config, ok := configIface.(LineConfig)
	if !ok {
		return fmt.Errorf("invalid config")
	}
	form := url.Values{}
	form.Add("message", fmt.Sprintf("%s\n%s", title, message))
	body := strings.NewReader(form.Encode())
	req, err := http.NewRequest("POST", LINE_API_URL, body)
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", config.Token))
	res, err := http.DefaultClient.Do(req)
	if err != nil {
		return err
	}
	if res.StatusCode < 200 || res.StatusCode >= 300 {
		return fmt.Errorf("line: %s", res.Status)
	}

	return nil
}
