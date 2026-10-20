# Guía de OctoOps

Documento de referencia de la aplicación: flujo de trabajo, autenticación, contratos
de datos y endpoints.

---

## 1. Visión general

OctoOps monitorea servidores e IoT desde un solo lugar. Un **agente** (binario Go) se
instala en cada máquina/dispositivo, recolecta métricas y las envía a un **backend**
(Rust + Postgres). El **usuario** las ve en un dashboard web.

DOMAIN_URL = https://octoops.apeiriumx.com

```
                    ┌────────────┐
  usuario ─────────▶│  dashboard │────────▶ GET /servers, /servers/:id, /servers/:id/metrics
  (navegador)       │  (cliente) │
                    └────────────┘
                          │ HTTPS (mismo origen)
                    ┌─────▼─────┐
                    │  backend  │◀─────── POST /servers/agents/report (firmado)
                    │   (Rust)  │
                    └─────┬─────┘
                          │
                    ┌─────▼─────┐
                    │  Postgres │
                    └───────────┘

  agente (Go) ──▶ instala ──▶ registra ──▶ reporta (cada N segundos)
```

---

## 2. Flujo de trabajo (end-to-end)

### 2.1 Login (usuario)

1. El dashboard (o tú directamente) visita `GET /auth/github/login`.
2. Redirige a GitHub OAuth.
3. GitHub redirige a `GET /auth/github/callback?code=…`.
4. El backend intercambia el `code`, crea/actualiza el usuario y setea cookies
   `access_token` (30 min) y `refresh_token` (7 días).
5. Redirige al `FRONTEND_URL`. El dashboard llama `GET /auth/me` para obtener el usuario.

### 2.2 Crear un servidor

1. El usuario (con JWT) hace `POST /servers` con `{ "name": "mi-vps" }`.
2. El backend crea un server en estado `pending` + un **token de enrollment**
   (aleatorio, hash SHA-256 guardado, un solo uso, 24 h).
3. Responde un comando de instalación con ese token.

### 2.3 Instalar el agente

1. El usuario pega el comando en la máquina destino:
   ```bash
   curl -fsSL https://octoops.apeiriumx.com/servers/install/<token> | sh
   ```
2. El script crea `OctoAgent/`, descarga el binario (`GET /servers/install/bin`) y
   escribe un `config.json` con `{ baseUrl, intervalSeconds, token, name, routes }`.

### 2.4 Registrar el agente

1. El agente arranca y ve el `token` → modo registro.
2. Genera un keypair **Ed25519**.
3. `POST /servers/agents/register` con `{ token, name, public_key, interval_seconds }`.
4. El backend valida el token, marca el server `active`, guarda la `public_key`,
   invalida el token y responde `{ "server_id": "…" }`.
5. El agente reescribe `config.json` (borra `token`/`name`, agrega `serverId`) y guarda
   su clave privada en `agent.key` (`0600`).

### 2.5 Reportar métricas

1. Cada `intervalSeconds`, el agente recolecta métricas (CPU, memoria, …).
2. Firma `timestamp + "\n" + body` con su clave privada.
3. `POST /servers/agents/report` con headers `X-Agent-ID`, `X-Timestamp`, `X-Signature`.
4. El backend verifica la firma (y la ventana anti-replay), valida el payload,
   lo guarda en `metrics` y actualiza `last_seen_at`.

### 2.6 Leer métricas

El dashboard lista los servidores y consulta sus métricas (endpoints 5.x).

---

## 3. Autenticación

| Mecanismo | Quién | Cómo |
|---|---|---|
| **GitHub OAuth + JWT (cookies)** | Usuario (dashboard web) | Cookies `httpOnly`, `SameSite=Lax`, `Path=/` |
| **Token de enrollment** | Agente (solo registro) | Token aleatorio de un solo uso, hash en DB |
| **Firma Ed25519** | Agente (reportes) | Firma `timestamp + "\n" + body` |

**Cookies:**

| Cookie | Duración | Uso |
|---|---|---|
| `access_token` | 30 min | autoriza peticiones del usuario |
| `refresh_token` | 7 días | rota el access (un solo uso) |

> Para un cliente **no-navegador** (app móvil, CLI, otro backend), las cookies no
> aplican: haría falta un auth por token (`Authorization: Bearer …`). No implementado aún.

---

## 4. Contratos de datos

### 4.1 Reporte de métricas (`contracts/report.md`)

```text
report = {
    "category_name": {
        "subcategory_name": number | bool
    }
}
```

Ejemplo:

```json
{
  "cpu":    { "percent": 12.5 },
  "memory": { "total_bytes": 7900, "used_bytes": 3100, "free_bytes": 4800 },
  "service":{ "running": true }
}
```

### 4.2 Usuario

```json
{ "id": "uuid", "username": "octocat", "email": "a@b.com", "avatar_url": "https://…" }
```

### 4.3 Servidor

```json
{
  "id": "uuid",
  "name": "mi-vps",
  "status": "online",
  "interval_seconds": 10,
  "last_seen_at": "2026-10-20T03:36:14Z",
  "created_at": "2026-10-20T03:36:14Z"
}
```

`status`:

| Valor | Significado |
|---|---|
| `online` | reportó dentro de `interval_seconds * 3` |
| `offline` | no reportó dentro de `interval_seconds * 3` |
| `unreported` | nunca reportó |

### 4.4 Métrica (historial)

```json
{ "recorded_at": "2026-10-20T03:36:14Z", "metrics": { "cpu": { "percent": 55.5 } } }
```

---

## 5. Endpoints

### Auth (cookies)

| Método | Ruta | Auth | Descripción |
|---|---|---|---|
| `GET` | `/auth/github/login` | público | Redirige a GitHub. |
| `GET` | `/auth/github/callback` | público | Completa el login, setea cookies. |
| `POST` | `/auth/refresh` | cookie `refresh_token` | Rota el refresh, emite tokens nuevos. |
| `GET` | `/auth/me` | cookie `access_token` | Devuelve el usuario actual. |
| `POST` | `/auth/logout` | cookie | Revoca el refresh y limpia cookies. |

### Servers

| Método | Ruta | Auth | Descripción |
|---|---|---|---|
| `GET` | `/servers` | JWT | Lista los servidores del usuario. |
| `POST` | `/servers` | JWT | Crea un server + token de enrollment. |
| `GET` | `/servers/:id` | JWT | Detalle de un servidor. |
| `GET` | `/servers/:id/metrics` | JWT | Historial de métricas. |
| `GET` | `/servers/install/:token` | token | Script de instalación. |
| `GET` | `/servers/install/bin` | público | Binario del agente. |
| `POST` | `/servers/agents/register` | token | Activa el server con su `public_key`. |

### Report

| Método | Ruta | Auth | Descripción |
|---|---|---|---|
| `POST` | `/servers/agents/report` | firma Ed25519 | Recibe y persiste métricas. |

---

## 6. Detalle de cada endpoint

### `GET /auth/github/login`
→ `303` redirige a `https://github.com/login/oauth/authorize?…`.

### `GET /auth/github/callback?code=…`
→ `303` al `FRONTEND_URL` con cookies. `401` si falla GitHub.

### `POST /auth/refresh`
→ `204` con cookies nuevas. `401` si el refresh es inválido/vencido.

### `GET /auth/me`
→ `200`:

```json
{ "id": "uuid", "username": "octocat", "email": "a@b.com", "avatar_url": "https://…" }
```

`401` si no hay sesión.

### `GET /servers`
→ `200`:

```json
[
  {
    "id": "uuid",
    "name": "mi-vps",
    "status": "online",
    "interval_seconds": 10,
    "last_seen_at": "2026-10-20T03:36:14Z",
    "created_at": "2026-10-20T03:36:14Z"
  }
]
```

### `POST /servers`

Body:

```json
{ "name": "mi-vps" }
```

→ `200`:

```json
{ "install_command": "curl -fsSL https://octoops.apeiriumx.com/servers/install/<token> | sh" }
```

### `GET /servers/:id`
→ `200` (misma forma que un elemento de la lista). `404` si no existe/no es tuyo.

### `GET /servers/:id/metrics?limit=100`
→ `200`:

```json
[
  { "recorded_at": "2026-10-20T03:36:14Z", "metrics": { "cpu": { "percent": 55.5 } } }
]
```

- `limit`: opcional, default `100`, máximo `1000`.
- `404` si el server no existe/no es tuyo.

### `GET /servers/install/:token`
→ `200` (`text/x-shellscript`). `404` si el token es inválido/vencido/usado.

### `GET /servers/install/bin`
→ `200` (`application/octet-stream`), binario del agente.

### `POST /servers/agents/register`

Body:

```json
{ "token": "<token>", "name": "mi-vps", "public_key": "<hex>", "interval_seconds": 10 }
```

→ `200`:

```json
{ "server_id": "uuid" }
```

`401` token inválido/vencido · `409` ya registrado · `400` `public_key` vacío.

### `POST /servers/agents/report`

Headers:

| Header | Valor |
|---|---|
| `X-Agent-ID` | `server_id` (uuid) |
| `X-Timestamp` | Unix seconds |
| `X-Signature` | firma Ed25519 en hex |

Body: un reporte (contrato 4.1).

→ `204` reporte recibido y persistido. `400` headers/payload inválidos ·
`401` firma inválida / timestamp viejo / agente desconocido.

---

## 7. Firma Ed25519 (protocolo)

1. El agente firma: `signature = ed25519.sign(private_key, timestamp + "\n" + body)`.
2. Envía `X-Agent-ID`, `X-Timestamp`, `X-Signature` (hex).
3. El backend reconstruye el mensaje, verifica contra la `public_key` registrada y
   rechaza timestamps con más de **30 s** de desvío (anti-replay).

---

## 8. Token de enrollment

- Aleatorio (32 bytes), codificado en hex.
- En la DB se guarda **solo su hash SHA-256**.
- **Un solo uso** y expira a las **24 h**.

---

## 9. Códigos de estado

| Código | Significado |
|---|---|
| `200` | OK (respuesta con cuerpo) |
| `204` | OK sin cuerpo (reporte recibido, refresh, logout) |
| `303` | Redirect (login GitHub) |
| `400` | Body/headers inválidos |
| `401` | No autenticado / firma o token inválidos |
| `404` | No encontrado |
| `409` | Conflicto (server ya registrado) |
| `500` | Error interno (DB) |
