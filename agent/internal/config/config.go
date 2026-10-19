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
	BaseURL         string `json:"baseUrl"`
	IntervalSeconds int    `json:"intervalSeconds"`
	ServerID        string `json:"serverId,omitempty"`
	Token           string `json:"token,omitempty"`
	Name            string `json:"name,omitempty"`
	Routes          Routes `json:"routes"`
}

type Routes struct {
	Metrics  Route `json:"metrics"`
	Register Route `json:"register"`
}

type Route struct {
	Path   string `json:"path"`
	Method string `json:"method"`
}

// ResolveConfigPath busca config.json en el directorio de trabajo y en el del
// ejecutable, y devuelve la ruta encontrada.
func ResolveConfigPath(fileName string) (string, error) {
	for _, dir := range defaultDirs() {
		path := filepath.Join(dir, fileName)
		if _, err := os.Stat(path); err == nil {
			return path, nil
		}
	}

	return "", fmt.Errorf("no se encontro %s", fileName)
}

// LoadDefault resuelve y carga el config, devolviendo también su ruta.
func LoadDefault(fileName string) (Config, string, error) {
	path, err := ResolveConfigPath(fileName)
	if err != nil {
		return Config{}, "", err
	}

	cfg, err := Load(path)
	if err != nil {
		return Config{}, "", err
	}

	return cfg, path, nil
}

func defaultDirs() []string {
	var dirs []string

	if cwd, err := os.Getwd(); err == nil {
		dirs = append(dirs, cwd)
	}

	if executablePath, err := os.Executable(); err == nil {
		dirs = append(dirs, filepath.Dir(executablePath))
	}

	return dirs
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

// Save serializa y escribe el config en la ruta dada.
func Save(path string, cfg Config) error {
	data, err := json.MarshalIndent(cfg, "", "  ")
	if err != nil {
		return fmt.Errorf("no se pudo serializar config.json: %w", err)
	}

	if err := os.WriteFile(path, data, 0o644); err != nil {
		return fmt.Errorf("no se pudo escribir %s: %w", path, err)
	}

	return nil
}

func (c Config) MetricsEndpoint() (string, error) {
	endpoint, err := url.JoinPath(c.BaseURL, c.Routes.Metrics.Path)
	if err != nil {
		return "", fmt.Errorf("no se pudo construir la URL de metrics: %w", err)
	}

	return endpoint, nil
}

func (c Config) RegisterEndpoint() (string, error) {
	endpoint, err := url.JoinPath(c.BaseURL, c.Routes.Register.Path)
	if err != nil {
		return "", fmt.Errorf("no se pudo construir la URL de register: %w", err)
	}

	return endpoint, nil
}

func (c Config) MetricsInterval() time.Duration {
	return time.Duration(c.IntervalSeconds) * time.Second
}

func (c Config) MetricsMethod() string {
	return c.Routes.Metrics.Method
}

func (c Config) RegisterMethod() string {
	return c.Routes.Register.Method
}

func (c *Config) normalize() {
	c.BaseURL = strings.TrimSpace(c.BaseURL)
	c.ServerID = strings.TrimSpace(c.ServerID)
	c.Token = strings.TrimSpace(c.Token)
	c.Name = strings.TrimSpace(c.Name)

	c.Routes.Metrics.Path = strings.TrimSpace(c.Routes.Metrics.Path)
	c.Routes.Metrics.Method = strings.ToUpper(strings.TrimSpace(c.Routes.Metrics.Method))

	c.Routes.Register.Path = strings.TrimSpace(c.Routes.Register.Path)
	c.Routes.Register.Method = strings.ToUpper(strings.TrimSpace(c.Routes.Register.Method))
	if c.Routes.Register.Path == "" {
		c.Routes.Register.Path = "/servers/agents/register"
	}
	if c.Routes.Register.Method == "" {
		c.Routes.Register.Method = "POST"
	}
}

func (c Config) validate() error {
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
