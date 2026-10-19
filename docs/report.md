# Router `report`

Recibe las métricas que envía el agente.

## Endpoint

`POST /servers/agents/report`

## Request

- `Content-Type: application/json`
- Body: un reporte con la forma definida en [`contracts/report.md`](../contracts/report.md).

## Response

| Código | Caso |
|---|---|
| `204` | Reporte recibido. |

## Auth

Pendiente: firma Ed25519 (`X-Agent-ID`, `X-Timestamp`, `X-Signature`), fase 3.
