# Router `report`

Recibe las métricas que envía el agente.

## Endpoint

`POST /servers/agents/report`

## Request

- `Content-Type: application/json`
- Body: un reporte con la forma definida en [`contracts/report.md`](../contracts/report.md).

## Auth (firma Ed25519)

El agente firma `timestamp + "\n" + body` con su llave privada y envía los headers:

| Header | Valor |
|---|---|
| `X-Agent-ID` | `server_id` (uuid) |
| `X-Timestamp` | Unix seconds |
| `X-Signature` | firma en hex |

El server verifica la firma contra la `public_key` registrada y rechaza timestamps con más de 30 s de desvío (anti-replay).

## Response

| Código | Caso |
|---|---|
| `204` | Reporte recibido y persistido. |
| `400` | Headers faltantes, timestamp/firma mal formados, o payload inválido. |
| `401` | Firma inválida, timestamp viejo, o agente desconocido/inactivo. |
