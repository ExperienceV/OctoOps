# OctoOps Agent

Agente ligero para Linux que recolecta métricas del host (CPU y memoria) y las reporta periódicamente a la API de OctoOps por HTTP.

- Lenguaje: Go (`go 1.25`, ver `go.mod`).
- Solo soporta Linux (lee `/proc/stat` y `/proc/meminfo`).
- Sin dependencias externas: solo librería estándar.

## Cómo funciona

1. Al arrancar lee `config.json` ubicado **junto al ejecutable** (no junto al código fuente, sino junto al binario compilado).
2. Hace un primer envío de métricas inmediatamente.
3. Luego repite el envío cada `intervalSeconds` con un `ticker`.
4. Termina de forma limpia con `Ctrl+C` (`SIGINT`) o `SIGTERM`.
5. Si una recolección o un envío falla, lo registra en el log y reintenta en el siguiente intervalo (no se cae el proceso).

```text
main.go → config.LoadFromExecutableDir("config.json")
        → collector.Collect() (cpu + memoria)
        → reporter.Run(ctx) (POST periódico a baseUrl + routes.metrics.path)
```

## Estructura

```text
agent/
├── main.go                   # Entrada: carga config, crea reporter, maneja SIGINT/SIGTERM
├── config.json               # Ejemplo de configuración (no usar el token de ejemplo en producción)
├── contracts/contracts.go    # Contrato JSON enviado a la API
└── internal/
    ├── config/config.go      # Carga, normalización y validación de config.json
    ├── collector/            # Lectura de /proc/stat y /proc/meminfo
    │   ├── collector.go      # Orquesta CPUPercent + MemoryTotal/Used/Free
    │   ├── cpu.go            # % CPU por delta de tiempos con pausa de 500ms
    │   ├── memory.go         # Memoria en bytes (KiB * 1024)
    │   └── platform.go       # ensureLinux(): error si runtime.GOOS != "linux"
    └── reporter/reporter.go  # Cliente HTTP, ticker, header Authorization, log con token enmascarado
```

## Requisitos

- Linux con `/proc/stat` y `/proc/meminfo` disponibles.
- Go 1.25+ solo si vas a compilar desde fuente.
- Conectividad hacia la `baseUrl` de la API.
- Un `token` válido (debe coincidir con `AGENT_TOKEN` de la API).

## Configuración (`config.json`)

El archivo debe estar en el mismo directorio que el binario ejecutado. Ejemplo:

```json
{
  "token": "super-secret-token",
  "baseUrl": "https://octoops.com",
  "intervalSeconds": 10,
  "routes": {
    "metrics": {
      "path": "/metrics",
      "method": "POST"
    }
  }
}
```

| Campo | Obligatorio | Descripción |
|---|---|---|
| `token` | Sí | Se envía como `Authorization: Bearer <token>`. No debe ir vacío ni con solo espacios. |
| `baseUrl` | Sí | URL base de la API, con esquema y host (ej. `http://localhost:3000`). Se unen `baseUrl + path` con `url.JoinPath`. |
| `intervalSeconds` | Sí | Cada cuántos segundos se envían métricas. Debe ser `> 0`. |
| `routes.metrics.path` | Sí | Ruta del endpoint de métricas (ej. `/metrics`). |
| `routes.metrics.method` | Sí | Método HTTP (ej. `POST`). Se normaliza a mayúsculas. |

Validaciones al arrancar (ver `internal/config/config.go`):

- `token`, `baseUrl`, `routes.metrics.path`, `routes.metrics.method` no vacíos.
- `baseUrl` debe tener esquema y host válidos.
- `intervalSeconds > 0`.
- Si algo falla, el agente termina con `error: ...` en `stderr` y código de salida `1`. Casos típicos:
  - `no se encontro <ruta>: ...` → falta `config.json` junto al ejecutable.
  - `config.json invalido: ...` → JSON mal formado.
  - `config.json requiere token / baseUrl / ...` → campo faltante o inválido.

## Compilar y ejecutar

Desde `agent/`:

```bash
go build -o agent .
```

Esto genera el binario `agent`. Luego coloca `config.json` junto a él:

```bash
ls -l
# agent
# config.json

./agent
```

Salida esperada:

```text
2026/09/28 12:00:00 Started
2026/09/28 12:00:00 Sending metrics request token=sup.........ken payload={"cpu":{"percent":12.5},"memory":{"total":...,"used":...,"free":...}}
```

Para detener: `Ctrl+C` o `kill <pid>` (`SIGTERM`).

> Nota: `go run .` también funciona para desarrollo, pero en ese caso `config.json` se busca junto al binario temporal compilado por Go, así que es más fiable compilar primero con `go build` y ejecutar `./agent`.

## Qué envía

### Request

```http
POST {baseUrl}{routes.metrics.path}
Authorization: Bearer <token>
Content-Type: application/json
```

Ejemplo con la config de arriba: `POST https://octoops.com/metrics`.

- Timeout HTTP: `5s` por request (`http.Client{Timeout: 5s}`).
- Código `2xx` = éxito. Cualquier otro código solo deja `Algo a fallado` en el log y se reintenta en el siguiente tick.

### Payload (`contracts.Metrics`)

```json
{
  "cpu": { "percent": 12.5 },
  "memory": { "total": 16777216000, "used": 8388608000, "free": 8388608000 }
}
```

- `cpu.percent`: `float64`, porcentaje `0-100`. Se calcula como `(1 - idleDelta/totalDelta) * 100` leyendo `/proc/stat` dos veces con `500ms` entre lecturas.
- `memory.total/used/free`: `uint64`, **bytes** (el agente lee KiB de `/proc/meminfo` y multiplica por `1024`).
  - `total` = `MemTotal`.
  - `free` (disponible) = `MemAvailable` (con fallback a `MemFree` si no existe).
  - `used` = `total - disponible`.

Este contrato debe coincidir con `metricsSchema` de la API (`api/src/modules/server/contract.ts`): objeto con `cpu.percent: number` y `memory.total/used/free: number`, sin propiedades extra.

## Logs y seguridad

- El token nunca se imprime completo: se enmascara como `abc.........xyz` (`maskToken` en `reporter.go`). Si mide 6 caracteres o menos se muestra tal cual, así que usa tokens largos.
- Cada envío exitoso en construcción deja una línea `Sending metrics request token=... payload=...`.
- Los fallos de red, de construcción del request o de código HTTP no-2xx dejan `Algo a fallado` en el log (sin detalle). Para depurar, revisa conectividad, `baseUrl`, token de la API y que el endpoint acepte el contrato.
- No commitees un `config.json` con un token real. El `config.json` del repo es solo un ejemplo.

## Errores comunes

| Síntoma | Causa probable |
|---|---|
| `no se encontro .../config.json` | Ejecutaste el binario desde otra ruta sin copiar `config.json` junto a él. |
| `este agente solo soporta Linux por ahora` | Lo ejecutaste en macOS/Windows. Solo corre en Linux. |
| `no se pudo leer /proc/stat` o `/proc/meminfo` | Contenedor/entorno sin `/proc` montado. |
| `401 Ese token no sirve` (lado API) | `token` del agente ≠ `AGENT_TOKEN` de la API, o falta el esquema `Bearer`. |
| Sin logs de envío tras `Started` | Revisa `intervalSeconds` y que `Collect()` no esté fallando en silencio (el `send` retorna sin log si `Collect()` o `json.Marshal` fallan). |
