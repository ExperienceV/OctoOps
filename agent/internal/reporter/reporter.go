package reporter

import (
	"bytes"
	"context"
	"encoding/json"
	"io"
	"log"
	"net/http"
	"time"

	"github.com/ExperienceV/OctoOps/agent/contracts"
	"github.com/ExperienceV/OctoOps/agent/internal/config"
)

type MetricsCollector interface {
	Collect() (contracts.Metrics, error)
}

type Reporter struct {
	collector MetricsCollector
	client    *http.Client
	endpoint  string
	method    string
	interval  time.Duration
	authToken string
}

func New(cfg config.Config, collector MetricsCollector) (*Reporter, error) {
	endpoint, err := cfg.MetricsEndpoint()
	if err != nil {
		return nil, err
	}

	return &Reporter{
		collector: collector,
		client: &http.Client{
			Timeout: 5 * time.Second,
		},
		endpoint:  endpoint,
		method:    cfg.MetricsMethod(),
		interval:  cfg.MetricsInterval(),
		authToken: cfg.TokenHeader(),
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
		return
	}

	body, err := json.Marshal(payload)
	if err != nil {
		return
	}

	requestCtx, cancel := context.WithTimeout(ctx, r.client.Timeout)
	defer cancel()

	request, err := http.NewRequestWithContext(requestCtx, r.method, r.endpoint, bytes.NewReader(body))
	if err != nil {
		return
	}

	request.Header.Set("Content-Type", "application/json")
	request.Header.Set("Authorization", r.authToken)
	log.Printf("Sending metrics request token=%s payload=%s", r.authToken, string(body))

	response, err := r.client.Do(request)
	if err != nil {
		return
	}
	defer response.Body.Close()

	_, _ = io.Copy(io.Discard, response.Body)
}
