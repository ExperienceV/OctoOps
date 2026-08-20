package collector

import (
	"fmt"
	"os"
	"runtime"
)

func ensureLinux() error {
	if runtime.GOOS != "linux" {
		return fmt.Errorf("este agente solo soporta Linux por ahora")
	}

	return nil
}

func readFileIfExists(path string) string {
	content, err := os.ReadFile(path)
	if err != nil {
		return ""
	}

	return string(content)
}
