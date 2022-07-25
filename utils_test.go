package main

import (
	"errors"
	"math/rand"
	"os"
	"path"
	"regexp"
	"testing"
	"time"
)

func TestFormatDuration(t *testing.T) {
	tests := []struct {
		in  time.Duration
		out string
	}{
		{0, "0s"},
		{1 * time.Second, "1s"},
		{1 * time.Minute, "1m 0s"},
		{1*time.Minute + 30*time.Second, "1m 30s"},
		{1*time.Hour + 1*time.Minute + 30*time.Second, "1h 1m 30s"},
		{25*time.Hour + 1*time.Minute + 30*time.Second, "1d 1h 1m 30s"},
	}
	for _, test := range tests {
		if out := formatDuration(test.in); out != test.out {
			t.Errorf("formatDuration(%d) = %s, want %s", test.in, out, test.out)
			return
		}
	}
}

func TestGetContext(t *testing.T) {
	home, err := os.UserHomeDir()
	if err != nil {
		t.Fatal(err)
	}
	if err := os.Chdir(home); err != nil {
		t.Fatal(err)
	}
	expect := regexp.MustCompile(`^[a-zA-Z0-9._-]+@[a-zA-Z0-9._-]+:~$`)
	str, err := getContext()
	if err != nil {
		t.Fatal(err)
	}
	if !expect.MatchString(str) {
		t.Errorf("getContext() = %s, want regex %s", str, expect.String())
		return
	}
}

func TestTryConfig(t *testing.T) {
	rnd := MakeRandomStr(8)
	home, err := os.UserHomeDir()
	if err != nil {
		t.Fatal(err)
	}
	fp, err := os.Create(path.Join(home, rnd))
	if err != nil {
		t.Fatal(err)
	}
	defer func() {
		fp.Close()
		os.Remove(path.Join(home, rnd))
	}()
	fp.WriteString(`
backends: ["dummy"]
dummy:
  item: dummy
`)

	cfg, err := tryConfig(path.Join(home, rnd))
	if err != nil {
		t.Errorf("tryConfig(%s) = %s, want nil", path.Join(home, rnd), err)
		return
	}
	if cfg == nil {
		t.Errorf("tryConfig(%s) = nil, want non-nil", path.Join(home, rnd))
		return
	}
}

func MakeRandomStr(digit uint32) string {
	const letters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"

	b := make([]byte, digit)
	if _, err := rand.Read(b); err != nil {
		panic(errors.New("unexpected error..."))
	}

	var result string
	for _, v := range b {
		result += string(letters[int(v)%len(letters)])
	}
	return result
}
