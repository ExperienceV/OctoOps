package main

import (
	"context"
	"fmt"
	"log"
	"os"
	"os/signal"
	"path/filepath"
	"syscall"

	"github.com/ExperienceV/OctoOps/agent/internal/collector"
	"github.com/ExperienceV/OctoOps/agent/internal/config"
	"github.com/ExperienceV/OctoOps/agent/internal/enroll"
	"github.com/ExperienceV/OctoOps/agent/internal/reporter"
)

func main() {
	log.SetFlags(log.Ldate | log.Ltime)
	log.Println("Started")

	cfg, cfgPath, err := config.LoadDefault("config.json")
	exitOnError(err)

	// Modo registro: si hay token, nos registramos antes de empezar a reportar.
	if cfg.Token != "" {
		cfg, cfgPath, err = register(cfg, cfgPath)
		exitOnError(err)
	}

	if cfg.ServerID == "" {
		exitOnError(fmt.Errorf("config.json no tiene ni token ni serverId"))
	}

	keyPath := filepath.Join(filepath.Dir(cfgPath), "agent.key")
	privKey, err := enroll.LoadKey(keyPath)
	exitOnError(err)

	service, err := reporter.New(cfg, collector.New(), cfg.ServerID, privKey)
	exitOnError(err)

	ctx, stop := signal.NotifyContext(context.Background(), os.Interrupt, syscall.SIGTERM)
	defer stop()

	exitOnError(service.Run(ctx))
}

// register genera el par de llaves, se registra con el server y reescribe el config.
func register(cfg config.Config, cfgPath string) (config.Config, string, error) {
	keypair, err := enroll.GenerateKeypair()
	if err != nil {
		return cfg, cfgPath, err
	}

	registerPath := cfg.Routes.Register.Path
	serverID, err := enroll.Register(cfg.BaseURL, registerPath, cfg.Token, cfg.Name, keypair.PublicKeyHex, cfg.IntervalSeconds)
	if err != nil {
		return cfg, cfgPath, err
	}

	dir := filepath.Dir(cfgPath)
	if err := enroll.SaveKey(filepath.Join(dir, "agent.key"), keypair.PrivateKeySeedHex); err != nil {
		return cfg, cfgPath, err
	}

	cfg.ServerID = serverID
	cfg.Token = ""
	cfg.Name = ""

	if err := config.Save(cfgPath, cfg); err != nil {
		return cfg, cfgPath, err
	}

	log.Printf("Registrado con serverId=%s", serverID)

	return cfg, cfgPath, nil
}

func exitOnError(err error) {
	if err == nil {
		return
	}

	fmt.Fprintf(os.Stderr, "error: %v\n", err)
	os.Exit(1)
}
