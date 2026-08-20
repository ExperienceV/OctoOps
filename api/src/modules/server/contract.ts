export interface Metrics {
    cpu: CpuMetrics;
    memory: MemMetrics;
}

export interface CpuMetrics {
    percent: number;
    temp: number;
}

export interface MemMetrics {
    total: number;
    used: number;
    free: number;
}
