package main

import (
	"context"
	"fmt"
	"os"
	"os/signal"
	"syscall"

	"github.com/ExperienceV/OctoOps/agent/internal/collector"
	"github.com/ExperienceV/OctoOps/agent/internal/config"
	"github.com/ExperienceV/OctoOps/agent/internal/reporter"
)

func main() {
	cfg, err := config.LoadFromExecutableDir("config.json")
	exitOnError(err)

	service, err := reporter.New(cfg, collector.New())
	exitOnError(err)

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	exitOnError(service.Run(ctx))
}

func exitOnError(err error) {
	if err == nil {
		return
	}

	fmt.Fprintf(os.Stderr, "error: %v\n", err)
	os.Exit(1)
}
