package collector

import (
	"bufio"
	"errors"
	"fmt"
	"os"
	"strconv"
	"strings"
)

const kibToBytes = 1024

type memoryInfo struct {
	total     uint64
	available uint64
}

func MemoryTotal() (uint64, error) {
	info, err := readMemoryInfo()
	if err != nil {
		return 0, err
	}

	return info.total * kibToBytes, nil
}

func MemoryUsed() (uint64, error) {
	info, err := readMemoryInfo()
	if err != nil {
		return 0, err
	}

	return (info.total - info.available) * kibToBytes, nil
}

func MemoryFree() (uint64, error) {
	info, err := readMemoryInfo()
	if err != nil {
		return 0, err
	}

	return info.available * kibToBytes, nil
}

func readMemoryInfo() (memoryInfo, error) {
	if err := ensureLinux(); err != nil {
		return memoryInfo{}, err
	}

	file, err := os.Open("/proc/meminfo")
	if err != nil {
		return memoryInfo{}, fmt.Errorf("no se pudo leer /proc/meminfo: %w", err)
	}
	defer file.Close()

	var total uint64
	var available uint64
	var free uint64

	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		fields := strings.Fields(scanner.Text())
		if len(fields) < 2 {
			continue
		}

		key := strings.TrimSuffix(fields[0], ":")
		value, err := strconv.ParseUint(fields[1], 10, 64)
		if err != nil {
			return memoryInfo{}, fmt.Errorf("valor de memoria invalido %q: %w", fields[1], err)
		}

		switch key {
		case "MemTotal":
			total = value
		case "MemAvailable":
			available = value
		case "MemFree":
			free = value
		}
	}

	if err := scanner.Err(); err != nil {
		return memoryInfo{}, fmt.Errorf("no se pudo escanear /proc/meminfo: %w", err)
	}

	if total == 0 {
		return memoryInfo{}, errors.New("MemTotal no esta disponible")
	}

	if available == 0 {
		available = free
	}

	if available > total {
		return memoryInfo{}, errors.New("MemAvailable es mayor a MemTotal")
	}

	return memoryInfo{total: total, available: available}, nil
}
