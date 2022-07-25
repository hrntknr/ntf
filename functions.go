package main

import (
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"os/exec"
	"strconv"
	"strings"
	"time"

	"github.com/adrg/xdg"
	"github.com/hrntknr/ntf/backends"
	"github.com/spf13/cobra"
)

func doneFunc(cmd *cobra.Command, args []string) {
	cfg, err := getConfig()
	if err != nil {
		log.Fatal(err)
	}
	backends.OverrideConfigs(&cfg, cmd)
	var title string
	if cmd.Flags().Lookup("title").Changed {
		title = cmd.Flags().Lookup("title").Value.String()
	} else {
		title, err = getContext()
		if err != nil {
			log.Fatal(err)
		}
	}
	if len(args) < 1 {
		log.Fatal("no argument")
	}
	execCmd := exec.Command(args[0], args[1:]...)
	execCmd.Stdout = os.Stdout
	execCmd.Stderr = os.Stderr
	execCmd.Stdin = os.Stdin
	start := time.Now()
	execCmd.Run()
	elapsed := time.Since(start)
	var msg string
	status := execCmd.ProcessState.ExitCode() == 0
	if status {
		msg = fmt.Sprintf("`%s` success in %s", strings.Join(args, " "), formatDuration(elapsed))
	} else {
		msg = fmt.Sprintf("`%s` failed (code %d) in %s", strings.Join(args, " "), execCmd.ProcessState.ExitCode(), formatDuration(elapsed))
	}
	if err := backends.Send(cfg, title, msg, &status); err != nil {
		log.Fatal(err)
	}
}

func sendFunc(cmd *cobra.Command, args []string) {
	cfg, err := getConfig()
	if err != nil {
		log.Fatal(err)
	}
	backends.OverrideConfigs(&cfg, cmd)
	var title string
	if cmd.Flags().Lookup("title").Changed {
		title = cmd.Flags().Lookup("title").Value.String()
	} else {
		title, err = getContext()
		if err != nil {
			log.Fatal(err)
		}
	}
	if len(args) < 1 {
		log.Fatal("no argument")
	}
	if err := backends.Send(cfg, title, strings.Join(args, " "), nil); err != nil {
		log.Fatal(err)
	}
}

func shellIntegrationFunc(cmd *cobra.Command, args []string) {
	shellPath := xdg.DataHome + "/ntf/ntf-shell-hook.sh"
	if _, err := os.Stat(shellPath); os.IsNotExist(err) {
		if err := os.MkdirAll(xdg.DataHome+"/ntf", 0755); err != nil {
			log.Fatal(err)
		}
		if err := ioutil.WriteFile(shellPath, ntf_shell_hook, 0755); err != nil {
			log.Fatal(err)
		}
	}
	fmt.Println(
		strings.Join([]string{
			"export AUTO_NTF_DONE_LONGER_THAN=${AUTO_NTF_DONE_LONGER_THAN:=10}",
			fmt.Sprintf("source %s", shellPath),
			"# To use ntf's shell integration, run this and add it to your shell's rc file:",
			"# eval \"$(ntf shell-integration)\"",
		}, "\n"),
	)
}

func shellDoneFunc(cmd *cobra.Command, args []string) {
	cfg, err := getConfig()
	if err != nil {
		log.Fatal(err)
	}
	backends.OverrideConfigs(&cfg, cmd)
	var title string
	if cmd.Flags().Lookup("title").Changed {
		title = cmd.Flags().Lookup("title").Value.String()
	} else {
		title, err = getContext()
		if err != nil {
			log.Fatal(err)
		}
	}
	if len(args) < 3 {
		log.Fatal("invalid arguments")
	}
	code, err := strconv.Atoi(args[0])
	if err != nil {
		log.Fatal(err)
	}
	duration, err := strconv.Atoi(args[1])
	if err != nil {
		log.Fatal(err)
	}
	command := strings.Join(args[2:], " ")
	status := code == 0
	if status {
		msg := fmt.Sprintf("`%s` success in %s", command, formatDuration(time.Second*time.Duration(duration)))
		if err := backends.Send(cfg, title, msg, &status); err != nil {
			log.Fatal(err)
		}
	} else {
		msg := fmt.Sprintf("`%s` failed (code %d) in %s", command, code, formatDuration(time.Second*time.Duration(duration)))
		if err := backends.Send(cfg, title, msg, &status); err != nil {
			log.Fatal(err)
		}
	}
}
