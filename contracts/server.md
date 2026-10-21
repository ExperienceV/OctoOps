# Contract of servers / devices (node)

Defines the shape of a monitored node as returned by the read endpoints
(`GET /servers`, `GET /servers/:id`). A node is either a **server** (with the Go agent)
or an **IoT device** (ESP32, etc.), distinguished by `kind`.

```text
node = {
    "id":               uuid,
    "name":             string,
    "kind":             "server" | "device",
    "status":           "online" | "offline" | "unreported",
    "interval_seconds": number | null,
    "last_seen_at":     timestamp | null,
    "created_at":       timestamp
}
```

## `kind`

| Value | Meaning | Auth |
|---|---|---|
| `server` | Linux server con el agente Go | Firma Ed25519 |
| `device` | Dispositivo IoT (ESP32, …) | `Authorization: Bearer <token>` |

## `status`

- `online`: reportó dentro de `interval_seconds * 3`.
- `offline`: no reportó dentro de `interval_seconds * 3`.
- `unreported`: nunca reportó.

## Example

```json
{
  "id": "d9ec7df5-d52f-4164-944d-1ad2fb4c016b",
  "name": "Incubadora",
  "kind": "device",
  "status": "online",
  "interval_seconds": 300,
  "last_seen_at": "2026-10-21T12:00:00Z",
  "created_at": "2026-10-21T10:00:00Z"
}
```
