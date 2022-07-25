package main

import (
	_ "embed"
	"log"

	"github.com/hrntknr/ntf/backends"
	"github.com/spf13/cobra"
)

func main() {
	if err := _main(); err != nil {
		log.Fatal(err)
	}
}

//go:embed ntf-shell-hook.sh
var ntf_shell_hook []byte

func _main() error {
	cmd := &cobra.Command{}
	cmd.CompletionOptions.DisableDefaultCmd = true

	doneCmd := &cobra.Command{
		Use:   "done",
		Short: "Execute the command and notify the message",
		Run:   doneFunc,
	}
	doneCmd.Flags().StringP("title", "t", "", "override title")

	sendCmd := &cobra.Command{
		Use:   "send",
		Short: "send notification",
		Run:   sendFunc,
	}
	sendCmd.Flags().StringP("title", "t", "", "override title")

	shellDoneCmd := &cobra.Command{
		Use:    "shell-done",
		Hidden: true,
		Run:    shellDoneFunc,
	}
	shellDoneCmd.Flags().StringP("title", "t", "", "override title")

	shellIntegrationCmd := &cobra.Command{
		Use:   "shell-integration",
		Short: "shell-integration",
		Run:   shellIntegrationFunc,
	}

	backends.RegisterFlags(doneCmd, sendCmd, shellIntegrationCmd, shellDoneCmd)
	cmd.AddCommand(doneCmd, sendCmd, shellIntegrationCmd, shellDoneCmd)
	return cmd.Execute()
}
