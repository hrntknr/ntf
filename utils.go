package main

import (
	"fmt"
	"log"
	"os"
	"os/user"
	"strings"
	"time"

	"github.com/adrg/xdg"
	"gopkg.in/yaml.v2"
)

func getConfig() (map[string]interface{}, error) {
	homeDir, err := os.UserHomeDir()
	if err != nil {
		return nil, err
	}
	configPath := []string{
		homeDir + "/.ntf.yml",
		xdg.ConfigHome + "/.ntf.yml",
	}
	for _, path := range configPath {
		cfg, err := tryConfig(path)
		if err != nil {
			log.Fatal(err)
		}
		if cfg != nil {
			return cfg, nil
		}
	}
	return map[string]interface{}{}, nil
}

func tryConfig(configPath string) (map[string]interface{}, error) {
	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		return nil, nil
	}
	fp, err := os.Open(configPath)
	if err != nil {
		return nil, err
	}
	defer fp.Close()

	var config map[string]interface{}
	if err := yaml.NewDecoder(fp).Decode(&config); err != nil {
		return nil, err
	}

	return config, nil
}

func getContext() (string, error) {
	user, err := user.Current()
	if err != nil {
		return "", err
	}
	hostname, err := os.Hostname()
	if err != nil {
		return "", err
	}
	path, err := os.Getwd()
	if err != nil {
		return "", err
	}
	homeDir, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	if strings.HasPrefix(path, homeDir) {
		path = "~" + path[len(homeDir):]
	}
	return fmt.Sprintf("%s@%s:%s", user.Username, hostname, path), nil
}

func formatDuration(d time.Duration) string {
	if d.Seconds() < 60 {
		return fmt.Sprintf("%ds", int(d.Seconds()))
	}
	if d.Minutes() < 60 {
		return fmt.Sprintf("%dm %ds", int(d.Minutes()), int(d.Seconds())-int(d.Minutes())*60)
	}
	if d.Hours() < 24 {
		return fmt.Sprintf("%dh %dm %ds", int(d.Hours()), int(d.Minutes())-int(d.Hours())*60, int(d.Seconds())-int(d.Minutes())*60)
	}
	return fmt.Sprintf("%dd %dh %dm %ds", int(d.Hours())/24, int(d.Hours())-int(d.Hours())/24*24, int(d.Minutes())-int(d.Hours())*60, int(d.Seconds())-int(d.Minutes())*60)
}
