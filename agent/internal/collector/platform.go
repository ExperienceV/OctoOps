package collector

import (
	"fmt"
	"runtime"
)

func ensureLinux() error {
	if runtime.GOOS != "linux" {
		return fmt.Errorf("este agente solo soporta Linux por ahora")
	}

	return nil
}
