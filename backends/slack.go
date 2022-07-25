package backends

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
)

func init() {
	backends["slack"] = NewSlack()
}

type SlackConfig struct {
	Webhook string  `mapstructure:"webhook" validate:"required"`
	Color   *string `mapstructure:"color" validate:"omitempty,hexcolor"`
}

type Slack struct {
}

func NewSlack() BackendInterface {
	return &Slack{}
}

func (*Slack) GetConfig() interface{} {
	return SlackConfig{}
}

func (*Slack) Send(configIface interface{}, title string, message string, status *bool) error {
	config, ok := configIface.(SlackConfig)
	if !ok {
		return fmt.Errorf("invalid config")
	}
	var color string
	if config.Color != nil {
		color = *config.Color
	} else if status != nil {
		if *status {
			color = "good"
		} else {
			color = "danger"
		}
	} else {
		color = "#ffffff"
	}

	body := map[string]interface{}{
		"attachments": []map[string]interface{}{
			{
				"title": title,
				"text":  message,
				"color": color,
			},
		},
	}
	jsonBody, err := json.Marshal(body)
	if err != nil {
		return err
	}
	res, err := http.Post(config.Webhook, "application/json", bytes.NewBuffer(jsonBody))
	if err != nil {
		return err
	}
	if res.StatusCode < 200 || res.StatusCode >= 300 {
		return fmt.Errorf("slack: %s", res.Status)
	}

	return nil
}
