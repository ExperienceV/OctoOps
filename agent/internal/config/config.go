package config

import (
	"encoding/json"
	"errors"
	"fmt"
	"net/url"
	"os"
	"path/filepath"
	"strings"
	"time"
)

type Config struct {
	Token           string `json:"token"`
	BaseURL         string `json:"baseUrl"`
	IntervalSeconds int    `json:"intervalSeconds"`
	Routes          Routes `json:"routes"`
}

type Routes struct {
	Metrics Route `json:"metrics"`
}

type Route struct {
	Path   string `json:"path"`
	Method string `json:"method"`
}

func LoadFromExecutableDir(fileName string) (Config, error) {
	configPath, err := resolveConfigPath(fileName)
	if err != nil {
		return Config{}, err
	}

	return Load(configPath)
}

func resolveConfigPath(fileName string) (string, error) {
	executablePath, err := os.Executable()
	if err != nil {
		return "", fmt.Errorf("no se pudo resolver la ruta del ejecutable: %w", err)
	}

	return filepath.Join(filepath.Dir(executablePath), fileName), nil
}

func Load(path string) (Config, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			return Config{}, fmt.Errorf("no se encontro %s: %w", path, err)
		}

		return Config{}, fmt.Errorf("no se pudo leer %s: %w", path, err)
	}

	var cfg Config
	if err := json.Unmarshal(content, &cfg); err != nil {
		return Config{}, fmt.Errorf("config.json invalido: %w", err)
	}

	cfg.normalize()

	if err := cfg.validate(); err != nil {
		return Config{}, err
	}

	return cfg, nil
}

func (c Config) MetricsEndpoint() (string, error) {
	endpoint, err := url.JoinPath(c.BaseURL, c.Routes.Metrics.Path)
	if err != nil {
		return "", fmt.Errorf("no se pudo construir la URL de metrics: %w", err)
	}

	return endpoint, nil
}

func (c Config) MetricsInterval() time.Duration {
	return time.Duration(c.IntervalSeconds) * time.Second
}

func (c Config) MetricsMethod() string {
	return c.Routes.Metrics.Method
}

func (c Config) TokenHeader() string {
	return "Bearer " + c.Token
}

func (c *Config) normalize() {
	c.Token = strings.TrimSpace(c.Token)
	c.BaseURL = strings.TrimSpace(c.BaseURL)
	c.Routes.Metrics.Path = strings.TrimSpace(c.Routes.Metrics.Path)
	c.Routes.Metrics.Method = strings.ToUpper(strings.TrimSpace(c.Routes.Metrics.Method))
}

func (c Config) validate() error {
	if c.Token == "" {
		return fmt.Errorf("config.json requiere token")
	}

	if c.BaseURL == "" {
		return fmt.Errorf("config.json requiere baseUrl")
	}

	parsedURL, err := url.Parse(c.BaseURL)
	if err != nil || parsedURL.Scheme == "" || parsedURL.Host == "" {
		return fmt.Errorf("config.json requiere un baseUrl valido")
	}

	if c.IntervalSeconds <= 0 {
		return fmt.Errorf("config.json requiere intervalSeconds mayor a 0")
	}

	if c.Routes.Metrics.Path == "" {
		return fmt.Errorf("config.json requiere routes.metrics.path")
	}

	if c.Routes.Metrics.Method == "" {
		return fmt.Errorf("config.json requiere routes.metrics.method")
	}

	return nil
}
