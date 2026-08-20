export interface Metrics {
    cpu: CpuMetrics;
    memory: MemMetrics;
}

export const metricsSchema = {
    type: "object",
    required: ["cpu", "memory"],
    additionalProperties: false,
    properties: {
        cpu: {
            type: "object",
            required: ["percent", "temp"],
            additionalProperties: false,
            properties: {
                percent: { type: "number" },
                temp: { type: "number" },
            },
        },
        memory: {
            type: "object",
            required: ["total", "used", "free"],
            additionalProperties: false,
            properties: {
                total: { type: "number" },
                used: { type: "number" },
                free: { type: "number" },
            },
        },
    },
} as const;

export interface CpuMetrics {
    percent: number;
    temp: number;
}

export interface MemMetrics {
    total: number;
    used: number;
    free: number;
}
