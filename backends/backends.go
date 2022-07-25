package backends

import (
	"fmt"
	"log"
	"reflect"

	"github.com/go-playground/validator/v10"
	"github.com/mitchellh/mapstructure"
	"github.com/spf13/cobra"
)

type BackendInterface interface {
	GetConfig() interface{}
	Send(config interface{}, title string, message string, status *bool) error
}

var backends = make(map[string]BackendInterface)
var validate = validator.New()

func RegisterFlags(cmd ...*cobra.Command) {
	for _, c := range cmd {
		c.Flags().StringSlice("backends", []string{}, "")
	}
	for backendKey, b := range backends {
		t := reflect.TypeOf(b.GetConfig())
		for i := 0; i < t.NumField(); i++ {
			registerFlag(backendKey, t.Field(i).Tag.Get("mapstructure"), t.Field(i), cmd...)
		}
	}
}

func registerFlag(backendKey string, key string, field reflect.StructField, cmd ...*cobra.Command) {
	kind := field.Type.Kind()
	if kind == reflect.Ptr {
		kind = field.Type.Elem().Kind()
	}
	switch kind {
	case reflect.String:
		for _, c := range cmd {
			c.Flags().String(fmt.Sprintf("%s.%s", backendKey, key), "", "")
		}
	case reflect.Int:
		for _, c := range cmd {
			c.Flags().Int(fmt.Sprintf("%s.%s", backendKey, key), 0, "")
		}
	}
}

func OverrideConfigs(cfg *map[string]interface{}, cmd *cobra.Command) {
	bflag := cmd.Flags().Lookup("backends")
	var bs []interface{}
	if bflag.Changed {
		val, err := cmd.Flags().GetStringSlice("backends")
		if err != nil {
			log.Fatal(err)
		}
		for _, v := range val {
			bs = append(bs, v)
		}
		(*cfg)["backends"] = bs
	}
	for backendKey, b := range backends {
		if (*cfg)[backendKey] == nil {
			(*cfg)[backendKey] = map[interface{}]interface{}{}
		}
		t := reflect.TypeOf(b.GetConfig())
		for i := 0; i < t.NumField(); i++ {
			overrideConfig((*cfg)[backendKey], backendKey, t.Field(i).Tag.Get("mapstructure"), t.Field(i), cmd)
		}
	}
}

func overrideConfig(cfg interface{}, backendKey string, key string, field reflect.StructField, cmd *cobra.Command) {
	kind := field.Type.Kind()
	if kind == reflect.Ptr {
		kind = field.Type.Elem().Kind()
	}
	switch kind {
	case reflect.String:
		v := cmd.Flags().Lookup(fmt.Sprintf("%s.%s", backendKey, key))
		if !v.Changed {
			return
		}
		val, err := cmd.Flags().GetString(fmt.Sprintf("%s.%s", backendKey, key))
		if err != nil {
			log.Fatal(err)
		}
		cfg.(map[interface{}]interface{})[key] = val
	case reflect.Int:
		v := cmd.Flags().Lookup(fmt.Sprintf("%s.%s", backendKey, key))
		if !v.Changed {
			return
		}
		val, err := cmd.Flags().GetInt(fmt.Sprintf("%s.%s", backendKey, key))
		if err != nil {
			log.Fatal(err)
		}
		cfg.(map[interface{}]interface{})[key] = val
	}
}

func Send(cfg map[string]interface{}, title string, msg string, status *bool) error {
	backendIfaces, ok := cfg["backends"]
	if !ok {
		return fmt.Errorf("no backends configured")
	}
	backendKeysIface, ok := backendIfaces.([]interface{})
	if !ok {
		return fmt.Errorf("backends is not a list")
	}
	for _, backendKeyIface := range backendKeysIface {
		backendKey, ok := backendKeyIface.(string)
		if !ok {
			log.Println(fmt.Errorf("backend is not a string"))
			continue
		}
		b, ok := backends[backendKey]
		if !ok {
			log.Println(fmt.Errorf("backend %s not found", backendKey))
			continue
		}
		cfgStruct := b.GetConfig()
		if err := mapstructure.Decode(cfg[backendKey], &cfgStruct); err != nil {
			log.Println(fmt.Errorf("error decoding backend %s config: %s", backendKey, err))
			continue
		}
		if err := validate.Struct(cfgStruct); err != nil {
			log.Println(fmt.Errorf("error validating backend %s config: %s", backendKey, err))
			continue
		}

		if err := b.Send(cfgStruct, title, msg, status); err != nil {
			return err
		}
	}
	return nil
}
