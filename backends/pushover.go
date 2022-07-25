package backends

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
)

func init() {
	backends["pushover"] = NewPushover()
}

const PUSHOVER_API_TOKEN = "abughxjjtuofgt89bz21mibut67j5t"
const PUSHOVER_API_URL = "https://api.pushover.net/1/messages.json"

type PushoverConfig struct {
	UserKey  string  `mapstructure:"user_key" validate:"required"`
	Device   *string `mapstructure:"device" validate:"omitempty"`
	Priority *string `mapstructure:"priority" validate:"omitempty,oneof=emergency high normal low lowest"`
	Retry    *int    `mapstructure:"retry" validate:"omitempty,min=30"`
	Expire   *int    `mapstructure:"expire" validate:"omitempty,min=0,max=10800"`
}

type Pushover struct {
}

func NewPushover() BackendInterface {
	return &Pushover{}
}

func (*Pushover) GetConfig() interface{} {
	return PushoverConfig{}
}

func (*Pushover) Send(configIface interface{}, title string, message string, status *bool) error {
	config, ok := configIface.(PushoverConfig)
	if !ok {
		return fmt.Errorf("invalid config")
	}

	body := map[string]interface{}{
		"token":   PUSHOVER_API_TOKEN,
		"user":    config.UserKey,
		"title":   title,
		"message": message,
	}
	if config.Device != nil {
		body["device"] = *config.Device
	}
	if config.Priority != nil {
		switch *config.Priority {
		case "emergency":
			body["priority"] = 2
		case "high":
			body["priority"] = 1
		case "normal":
			body["priority"] = 0
		case "low":
			body["priority"] = -1
		case "lowest":
			body["priority"] = -2
		}
	}
	if config.Retry != nil {
		body["retry"] = *config.Retry
	}
	if config.Expire != nil {
		body["expire"] = *config.Expire
	}
	jsonBody, err := json.Marshal(body)
	if err != nil {
		return err
	}
	res, err := http.Post(PUSHOVER_API_URL, "application/json", bytes.NewBuffer(jsonBody))
	if err != nil {
		return err
	}
	if res.StatusCode < 200 || res.StatusCode >= 300 {
		return fmt.Errorf("pushover: %s", res.Status)
	}
	return nil
}
