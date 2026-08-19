## General

All internal TypeScript properties use `camelCase`.

Examples:

- `cpuPercent`
- `memoryPercent`
- `diskUsageBytes`
- `uptimeSeconds`

Do not use alternative names for the same concept.

Incorrect:

- `percentOfCpu`
- `cpuUsagePercent`
- `cpu_percentage`
- `cpu_usage`

Correct:

- `cpuPercent`

## Units

Units must be included in the property name when necessary.

- `memoryUsedBytes`
- `diskFreeBytes`
- `uptimeSeconds`
- `temperatureCelsius`

Percentages always use the `Percent` suffix:

- `cpuPercent`
- `memoryPercent`
- `diskPercent`