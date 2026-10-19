package reporter

import (
	"bytes"
	"context"
	"crypto/ed25519"
	"encoding/hex"
	"encoding/json"
	"io"
	"log"
	"net/http"
	"strconv"
	"time"

	"github.com/ExperienceV/OctoOps/agent/internal/config"
	"github.com/ExperienceV/OctoOps/agent/types"
)

type MetricsCollector interface {
	Collect() (types.Report, error)
}

type Reporter struct {
	collector MetricsCollector
	client    *http.Client
	endpoint  string
	method    string
	interval  time.Duration
	serverID  string
	privKey   ed25519.PrivateKey
}

func New(cfg config.Config, collector MetricsCollector, serverID string, privKey ed25519.PrivateKey) (*Reporter, error) {
	endpoint, err := cfg.MetricsEndpoint()
	if err != nil {
		return nil, err
	}

	return &Reporter{
		collector: collector,
		client: &http.Client{
			Timeout: 5 * time.Second,
		},
		endpoint: endpoint,
		method:   cfg.MetricsMethod(),
		interval: cfg.MetricsInterval(),
		serverID: serverID,
		privKey:  privKey,
	}, nil
}

func (r *Reporter) Run(ctx context.Context) error {
	r.send(ctx)

	ticker := time.NewTicker(r.interval)
	defer ticker.Stop()

	for {
		select {
		case <-ctx.Done():
			return nil
		case <-ticker.C:
			r.send(ctx)
		}
	}
}

func (r *Reporter) send(ctx context.Context) {
	payload, err := r.collector.Collect()
	if err != nil {
		log.Printf("Error recolectando métricas: %v", err)
		return
	}

	body, err := json.Marshal(payload)
	if err != nil {
		log.Printf("Error serializando métricas: %v", err)
		return
	}

	timestamp := strconv.FormatInt(time.Now().Unix(), 10)

	// Firma Ed25519 sobre: timestamp + "\n" + body
	message := append([]byte(timestamp+"\n"), body...)
	signature := ed25519.Sign(r.privKey, message)

	requestCtx, cancel := context.WithTimeout(ctx, r.client.Timeout)
	defer cancel()

	request, err := http.NewRequestWithContext(requestCtx, r.method, r.endpoint, bytes.NewReader(body))
	if err != nil {
		log.Println("Algo a fallado")
		return
	}

	request.Header.Set("Content-Type", "application/json")
	request.Header.Set("X-Agent-ID", r.serverID)
	request.Header.Set("X-Timestamp", timestamp)
	request.Header.Set("X-Signature", hex.EncodeToString(signature))

	log.Printf("Enviando reporte agent=%s ts=%s payload=%s", r.serverID, timestamp, string(body))

	response, err := r.client.Do(request)
	if err != nil {
		log.Println("Algo a fallado")
		return
	}
	defer response.Body.Close()

	if response.StatusCode < http.StatusOK || response.StatusCode >= http.StatusMultipleChoices {
		log.Printf("El server rechazó el reporte: %d", response.StatusCode)
	}

	_, _ = io.Copy(io.Discard, response.Body)
}
