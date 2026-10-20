package enroll

import (
	"bytes"
	"crypto/ed25519"
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"strings"
	"time"
)

// Keypair es un par de llaves Ed25519 generado por el agente.
type Keypair struct {
	PublicKeyHex       string
	PrivateKeySeedHex  string
}

// GenerateKeypair genera un par de llaves Ed25519.
func GenerateKeypair() (Keypair, error) {
	pub, priv, err := ed25519.GenerateKey(rand.Reader)
	if err != nil {
		return Keypair{}, fmt.Errorf("no se pudo generar el par de llaves: %w", err)
	}

	return Keypair{
		PublicKeyHex:      hex.EncodeToString(pub),
		PrivateKeySeedHex: hex.EncodeToString(priv.Seed()),
	}, nil
}

// Register envía el token + nombre + llave pública + intervalo al server y devuelve el server ID.
func Register(baseURL, registerPath, token, name, publicKeyHex string, intervalSeconds int) (string, error) {
	payload := struct {
		Token           string `json:"token"`
		Name            string `json:"name"`
		PublicKey       string `json:"public_key"`
		IntervalSeconds int    `json:"interval_seconds"`
	}{
		Token:           token,
		Name:            name,
		PublicKey:       publicKeyHex,
		IntervalSeconds: intervalSeconds,
	}

	body, err := json.Marshal(payload)
	if err != nil {
		return "", fmt.Errorf("no se pudo serializar el registro: %w", err)
	}

	endpoint := strings.TrimRight(baseURL, "/") + registerPath
	req, err := http.NewRequest(http.MethodPost, endpoint, bytes.NewReader(body))
	if err != nil {
		return "", fmt.Errorf("no se pudo crear la petición de registro: %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	client := &http.Client{Timeout: 10 * time.Second}
	resp, err := client.Do(req)
	if err != nil {
		return "", fmt.Errorf("falló el registro: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		data, _ := io.ReadAll(resp.Body)
		return "", fmt.Errorf("registro rechazado (%d): %s", resp.StatusCode, strings.TrimSpace(string(data)))
	}

	var result struct {
		ServerID string `json:"server_id"`
	}
	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return "", fmt.Errorf("respuesta de registro inválida: %w", err)
	}
	if result.ServerID == "" {
		return "", fmt.Errorf("respuesta de registro sin server_id")
	}

	return result.ServerID, nil
}

// SaveKey guarda la semilla de la llave privada en hex (permisos 0600).
func SaveKey(path, seedHex string) error {
	if err := os.WriteFile(path, []byte(seedHex+"\n"), 0o600); err != nil {
		return fmt.Errorf("no se pudo guardar la llave privada: %w", err)
	}

	return nil
}

// LoadKey lee la semilla de la llave privada y reconstruye la llave Ed25519.
func LoadKey(path string) (ed25519.PrivateKey, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("no se pudo leer la llave privada %s: %w", path, err)
	}

	seed, err := hex.DecodeString(strings.TrimSpace(string(data)))
	if err != nil {
		return nil, fmt.Errorf("llave privada inválida: %w", err)
	}

	if len(seed) != ed25519.SeedSize {
		return nil, fmt.Errorf("llave privada con tamaño incorrecto")
	}

	return ed25519.NewKeyFromSeed(seed), nil
}
