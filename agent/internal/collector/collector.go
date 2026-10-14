package collector

import "github.com/ExperienceV/OctoOps/agent/types"

type Collector struct{}

func New() Collector {
	return Collector{}
}

func (Collector) Collect() (types.Report, error) {
	cpuPercent, err := CPUPercent()
	if err != nil {
		return nil, err
	}

	memoryTotal, err := MemoryTotal()
	if err != nil {
		return nil, err
	}

	memoryUsed, err := MemoryUsed()
	if err != nil {
		return nil, err
	}

	memoryFree, err := MemoryFree()
	if err != nil {
		return nil, err
	}

	return types.Report{
		"cpu": {
			"percent": cpuPercent,
		},
		"memory": {
			"total_bytes": memoryTotal,
			"used_bytes":  memoryUsed,
			"free_bytes":  memoryFree,
		},
	}, nil
}
