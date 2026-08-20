package collector

import (
	"errors"
	"fmt"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
)

type temperatureCandidate struct {
	label       string
	temperature float64
	priority    int
}

func CPUTemp() (float64, error) {
	if err := ensureLinux(); err != nil {
		return 0, err
	}

	candidates := append(readHwmonTemperatures(), readThermalZoneTemperatures()...)
	if len(candidates) == 0 {
		return 0, errors.New("no se encontraron sensores de temperatura")
	}

	sort.Slice(candidates, func(i, j int) bool {
		if candidates[i].priority != candidates[j].priority {
			return candidates[i].priority > candidates[j].priority
		}

		return candidates[i].temperature > candidates[j].temperature
	})

	return candidates[0].temperature, nil
}

func readHwmonTemperatures() []temperatureCandidate {
	var candidates []temperatureCandidate

	hwmonDirs, err := filepath.Glob("/sys/class/hwmon/hwmon*")
	if err != nil {
		return candidates
	}

	for _, dir := range hwmonDirs {
		name := strings.TrimSpace(readFileIfExists(filepath.Join(dir, "name")))
		inputFiles, err := filepath.Glob(filepath.Join(dir, "temp*_input"))
		if err != nil {
			continue
		}

		for _, inputFile := range inputFiles {
			value, err := readTemperatureValue(inputFile)
			if err != nil {
				continue
			}

			labelFile := strings.TrimSuffix(inputFile, "_input") + "_label"
			label := strings.TrimSpace(readFileIfExists(labelFile))
			fullLabel := strings.TrimSpace(strings.Join([]string{name, label}, " "))
			if fullLabel == "" {
				fullLabel = inputFile
			}

			candidates = append(candidates, temperatureCandidate{
				label:       fullLabel,
				temperature: value,
				priority:    temperaturePriority(fullLabel),
			})
		}
	}

	return candidates
}

func readThermalZoneTemperatures() []temperatureCandidate {
	var candidates []temperatureCandidate

	zoneDirs, err := filepath.Glob("/sys/class/thermal/thermal_zone*")
	if err != nil {
		return candidates
	}

	for _, dir := range zoneDirs {
		value, err := readTemperatureValue(filepath.Join(dir, "temp"))
		if err != nil {
			continue
		}

		label := strings.TrimSpace(readFileIfExists(filepath.Join(dir, "type")))
		if label == "" {
			label = dir
		}

		candidates = append(candidates, temperatureCandidate{
			label:       label,
			temperature: value,
			priority:    temperaturePriority(label),
		})
	}

	return candidates
}

func readTemperatureValue(path string) (float64, error) {
	raw := strings.TrimSpace(readFileIfExists(path))
	if raw == "" {
		return 0, fmt.Errorf("archivo vacio: %s", path)
	}

	value, err := strconv.ParseFloat(raw, 64)
	if err != nil {
		return 0, fmt.Errorf("temperatura invalida en %s: %w", path, err)
	}

	switch {
	case value > 1000:
		return value / 1000, nil
	case value > 250:
		return value / 10, nil
	default:
		return value, nil
	}
}

func temperaturePriority(label string) int {
	label = strings.ToLower(label)

	switch {
	case strings.Contains(label, "x86_pkg_temp"):
		return 6
	case strings.Contains(label, "coretemp"), strings.Contains(label, "package"):
		return 5
	case strings.Contains(label, "cpu"), strings.Contains(label, "tctl"), strings.Contains(label, "tdie"):
		return 4
	case strings.Contains(label, "k10temp"), strings.Contains(label, "acpitz"):
		return 3
	default:
		return 1
	}
}
