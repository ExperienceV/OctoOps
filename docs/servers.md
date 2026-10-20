# Router `servers`

Gestión y lectura de los servidores (nodos) monitoreados.

## Endpoints

| Método | Ruta | Auth | Descripción |
|---|---|---|---|
| `GET` | `/servers` | JWT | Lista los servidores del usuario. |
| `POST` | `/servers` | JWT | Crea un servidor y devuelve el comando de instalación. |
| `GET` | `/servers/:id` | JWT | Detalle de un servidor. |
| `GET` | `/servers/:id/metrics` | JWT | Historial de métricas. |
| `GET` | `/servers/install/:token` | token | Script de instalación. |
| `GET` | `/servers/install/bin` | público | Binario del agente. |
| `POST` | `/servers/agents/register` | token | Activa el servidor con su llave pública. |

## Estado del servidor

Derivado de `last_seen_at` y `interval_seconds`:

- `online`: reportó dentro de `interval_seconds * 3`.
- `offline`: no reportó dentro de `interval_seconds * 3`.
- `unreported`: nunca reportó.

## GET /servers

Respuesta `200`:

```json
[
  {
    "id": "uuid",
    "name": "mi-vps",
    "status": "online",
    "interval_seconds": 10,
    "last_seen_at": "2026-10-19T20:50:06Z",
    "created_at": "2026-10-19T20:40:00Z"
  }
]
```

`401` si no hay sesión.

## POST /servers

Body: `{ "name": "mi-vps" }`.

Respuesta `200`:

```json
{ "install_command": "curl -fsSL https://octoops.com/servers/install/<token> | sh" }
```

## GET /servers/:id

Respuesta `200` (misma forma que un elemento de la lista) o `404` si no existe o no es del usuario.

## GET /servers/:id/metrics?limit=100

Respuesta `200`:

```json
[
  {
    "recorded_at": "2026-10-19T20:50:06Z",
    "metrics": { "cpu": { "percent": 42.5 }, "memory": { "total_bytes": 16432762880 } }
  }
]
```

- `limit`: opcional, default `100`, máximo `1000`.
- `404` si el servidor no existe o no es del usuario.

## GET /servers/install/:token

Devuelve el script de instalación (`text/x-shellscript`). `404` si el token es inválido, expirado o ya usado.

## GET /servers/install/bin

Devuelve el binario del agente (`application/octet-stream`).

## POST /servers/agents/register

Body:

```json
{ "token": "<token>", "name": "mi-vps", "public_key": "<hex>", "interval_seconds": 10 }
```

Respuesta `200`:

```json
{ "server_id": "uuid" }
```

`401` si el token es inválido o vencido, `409` si ya fue registrado.
