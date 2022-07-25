package backends

import (
	"bytes"
	"encoding/json"
	"fmt"
	"net/http"
)

func init() {
	backends["pushbullet"] = NewPushbullet()
}

const PUSHBULET_API_URL = "https://api.pushbullet.com/v2/pushes"

type PushbulletConfig struct {
	Token string  `mapstructure:"token" validate:"required"`
}

type Pushbullet struct {
}

func NewPushbullet() BackendInterface {
	return &Pushbullet{}
}

func (*Pushbullet) GetConfig() interface{} {
	return PushbulletConfig{}
}

func (*Pushbullet) Send(configIface interface{}, title string, message string, status *bool) error {
	config, ok := configIface.(PushbulletConfig)
	if !ok {
		return fmt.Errorf("invalid config")
	}
	body := map[string]interface{}{
		"type": "note",
		"title": title,
		"body": message,
	}
	jsonBody, err := json.Marshal(body)
	if err != nil {
		return err
	}
	req, err := http.NewRequest("POST", PUSHBULET_API_URL, bytes.NewBuffer(jsonBody))
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", config.Token))
	res, err := http.DefaultClient.Do(req)
	if err != nil {
		return err
	}
	if res.StatusCode < 200 || res.StatusCode >= 300 {
		return fmt.Errorf("pushbullet: %s", res.Status)
	}

	return nil
}
