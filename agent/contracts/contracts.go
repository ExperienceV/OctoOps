package contracts

type Metrics struct {
	CPU    CpuMetrics `json:"cpu"`
	Memory MemMetrics `json:"memory"`
}

type CpuMetrics struct {
	Percent float64 `json:"percent"`
}

type MemMetrics struct {
	Total uint64 `json:"total"`
	Used  uint64 `json:"used"`
	Free  uint64 `json:"free"`
}
