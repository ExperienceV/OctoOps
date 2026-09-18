package collector

import "github.com/ExperienceV/OctoOps/agent/contracts"

type Collector struct{}

func New() Collector {
	return Collector{}
}

func (Collector) Collect() (contracts.Metrics, error) {
	cpuPercent, err := CPUPercent()
	if err != nil {
		return contracts.Metrics{}, err
	}

	memoryTotal, err := MemoryTotal()
	if err != nil {
		return contracts.Metrics{}, err
	}

	memoryUsed, err := MemoryUsed()
	if err != nil {
		return contracts.Metrics{}, err
	}

	memoryFree, err := MemoryFree()
	if err != nil {
		return contracts.Metrics{}, err
	}

	return contracts.Metrics{
		CPU: contracts.CpuMetrics{
			Percent: cpuPercent,
		},
		Memory: contracts.MemMetrics{
			Total: memoryTotal,
			Used:  memoryUsed,
			Free:  memoryFree,
		},
	}, nil
}
