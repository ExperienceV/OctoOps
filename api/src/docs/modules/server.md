# Server Module (`/server`)

## Objective

This section represents Linux servers monitored by a custom agent.

At this stage, it should only handle the essentials:

- Register servers
- Identify each server with a unique `apiKey`
- Receive basic metrics
- Determine whether the server is reporting or not

## Current Scope

- Manually create servers
- List created servers
- Get a server by `id`
- Delete a server
- Receive agent metrics over HTTP
- Calculate status based on `lastSeenAt`

## Base Rules

- A server is created from the dashboard.
- Each server has a unique `apiKey`.
- The agent authenticates using the `X-API-Key` header.
- The backend assigns the metric timestamp.
- `status` must not be stored as a fixed truth.
- If `lastSeenAt` is `null`, the server status is `unreported`.
- If the latest report is within the expected threshold, it is `online`.
- If it exceeds the expected threshold, it is `offline`.
- Deleting a server must also delete its metrics.

## Minimum Data Models

### Server

- `id`
- `name`
- `hostname`
- `apiKeyHash`
- `lastSeenAt`
- `expectedReportIntervalSeconds`
- `createdAt`
- `updatedAt`

### Server Metric

- `id`
- `serverId`
- `cpuPercent`
- `memoryTotalBytes`
- `memoryUsedBytes`
- `memoryFreeBytes`
- `recordedAt`

## Agent Metrics

For now, the server agent reports:

- `cpu.percent`
- `memory.total`
- `memory.used`
- `memory.free`

## First Stage Endpoints

All under `/api/v1`.

- `GET /api/v1/servers`
- `POST /api/v1/servers`
- `GET /api/v1/servers/:id`
- `DELETE /api/v1/servers/:id`
- `POST /api/v1/servers/:id/metrics`

## Simple Ingestion Payload

```json
{
  "cpu": {
    "percent": 21.4
  },
  "memory": {
    "total": 8589934592,
    "used": 5767168000,
    "free": 2822766592
  }
}
```

## Server Status

Simple rule to start:

- `unreported`: Never sent metrics
- `online`: `lastSeenAt` is within the threshold
- `offline`: `lastSeenAt` exceeds the threshold

Recommended initial threshold:

- `90` seconds

## Initial `/server` View

For this stage, it is sufficient to display:

- Status
- Name
- Hostname
- Last report

## Note

There is no need to define alerts, advanced agent metadata, complex historical charts, or extra integrations yet. These can be added once the base module is working properly.
