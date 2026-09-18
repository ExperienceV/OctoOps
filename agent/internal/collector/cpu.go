package collector

import (
	"bufio"
	"errors"
	"fmt"
	"os"
	"strconv"
	"strings"
	"time"
)

type cpuTimes struct {
	idle  uint64
	total uint64
}

func CPUPercent() (float64, error) {
	if err := ensureLinux(); err != nil {
		return 0, err
	}

	first, err := readCPUTimes()
	if err != nil {
		return 0, err
	}

	time.Sleep(500 * time.Millisecond)

	second, err := readCPUTimes()
	if err != nil {
		return 0, err
	}

	idleDelta := float64(second.idle - first.idle)
	totalDelta := float64(second.total - first.total)
	if totalDelta <= 0 {
		return 0, errors.New("no fue posible calcular el uso de CPU")
	}

	return (1 - idleDelta/totalDelta) * 100, nil
}

func readCPUTimes() (cpuTimes, error) {
	file, err := os.Open("/proc/stat")
	if err != nil {
		return cpuTimes{}, fmt.Errorf("no se pudo leer /proc/stat: %w", err)
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	if !scanner.Scan() {
		if err := scanner.Err(); err != nil {
			return cpuTimes{}, fmt.Errorf("no se pudo escanear /proc/stat: %w", err)
		}

		return cpuTimes{}, errors.New("/proc/stat no contiene datos de CPU")
	}

	fields := strings.Fields(scanner.Text())
	if len(fields) < 8 || fields[0] != "cpu" {
		return cpuTimes{}, errors.New("formato inesperado en /proc/stat")
	}

	values := make([]uint64, 0, len(fields)-1)
	for _, field := range fields[1:] {
		value, err := strconv.ParseUint(field, 10, 64)
		if err != nil {
			return cpuTimes{}, fmt.Errorf("valor de CPU invalido %q: %w", field, err)
		}

		values = append(values, value)
	}

	var total uint64
	for _, value := range values {
		total += value
	}

	idle := values[3]
	if len(values) > 4 {
		idle += values[4]
	}

	return cpuTimes{idle: idle, total: total}, nil
}
