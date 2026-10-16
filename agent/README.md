# OctoOps Agent

Agente ligero para Linux que recolecta métricas del host (CPU y memoria) y las reporta periódicamente a la API de OctoOps por HTTP.

- Lenguaje: Go (`go 1.25`, ver `go.mod`).
- Solo soporta Linux (lee `/proc/stat` y `/proc/meminfo`).
- Sin dependencias externas: solo librería estándar.

## Cómo funciona

1. Al arrancar lee `config.json` buscándolo primero en el **directorio de trabajo actual** y, si no está, junto al ejecutable.
2. Hace un primer envío de métricas inmediatamente.
3. Luego repite el envío cada `intervalSeconds` con un `ticker`.
4. Termina de forma limpia con `Ctrl+C` (`SIGINT`) o `SIGTERM`.
5. Si una recolección o un envío falla, lo registra en el log y reintenta en el siguiente intervalo (no se cae el proceso).

```text
main.go → config.LoadDefault("config.json")
        → collector.Collect() (cpu + memoria)
        → reporter.Run(ctx) (POST periódico a baseUrl + routes.metrics.path)
```

## Estructura

```text
agent/
├── main.go                   # Entrada: carga config, crea reporter, maneja SIGINT/SIGTERM
├── config.json               # Ejemplo de configuración
├── types/report.go           # Contrato del payload (contracts/report.md)
└── internal/
    ├── config/config.go      # Carga, normalización y validación de config.json
    ├── collector/            # Lectura de /proc/stat y /proc/meminfo
    │   ├── collector.go      # Orquesta CPUPercent + MemoryTotal/Used/Free
    │   ├── cpu.go            # % CPU por delta de tiempos con pausa de 500ms
    │   ├── memory.go         # Memoria en bytes (KiB * 1024)
    │   └── platform.go       # ensureLinux(): error si runtime.GOOS != "linux"
    └── reporter/reporter.go  # Cliente HTTP, ticker, POST periódico de métricas
```

## Requisitos

- Linux con `/proc/stat` y `/proc/meminfo` disponibles.
- Go 1.25+ solo si vas a compilar desde fuente.
- Conectividad hacia la `baseUrl` de la API.

## Configuración (`config.json`)

El agente busca `config.json` primero en el directorio de trabajo actual y, si no está, junto al ejecutable. Ejemplo:

```json
{
  "baseUrl": "http://127.0.0.1:8000",
  "intervalSeconds": 10,
  "routes": {
    "metrics": {
      "path": "/servers/agents/report",
      "method": "POST"
    }
  }
}
```

| Campo | Obligatorio | Descripción |
|---|---|---|
| `baseUrl` | Sí | URL base de la API, con esquema y host (ej. `http://127.0.0.1:8000`). Se unen `baseUrl + path` con `url.JoinPath`. |
| `intervalSeconds` | Sí | Cada cuántos segundos se envían métricas. Debe ser `> 0`. |
| `routes.metrics.path` | Sí | Ruta del endpoint de métricas (ej. `/servers/agents/report`). |
| `routes.metrics.method` | Sí | Método HTTP (ej. `POST`). Se normaliza a mayúsculas. |

Validaciones al arrancar (ver `internal/config/config.go`):

- `baseUrl`, `routes.metrics.path`, `routes.metrics.method` no vacíos.
- `baseUrl` debe tener esquema y host válidos.
- `intervalSeconds > 0`.
- Si algo falla, el agente termina con `error: ...` en `stderr` y código de salida `1`. Casos típicos:
  - `no se encontro config.json` → no hay `config.json` en el directorio de trabajo ni junto al ejecutable.
  - `config.json invalido: ...` → JSON mal formado.
  - `config.json requiere baseUrl / ...` → campo faltante o inválido.

## Compilar y ejecutar

Desde `agent/`:

```bash
go run .
```

O compila el binario y ejecútalo (con `config.json` junto a él o en el directorio de trabajo):

```bash
go build -o agent .
./agent
```

Salida esperada:

```text
2026/09/28 12:00:00 Started
2026/09/28 12:00:00 Sending metrics request payload={"cpu":{"percent":12.5},"memory":{"total_bytes":...,"used_bytes":...,"free_bytes":...}}
```

Para detener: `Ctrl+C` o `kill <pid>` (`SIGTERM`).

## Qué envía

### Request

```http
POST {baseUrl}{routes.metrics.path}
Content-Type: application/json
```

- Timeout HTTP: `5s` por request (`http.Client{Timeout: 5s}`).
- Código `2xx` = éxito. Cualquier otro código solo deja `Algo a fallado` en el log y se reintenta en el siguiente tick.

### Payload (`types.Report`)

```json
{
  "cpu":    { "percent": 12.5 },
  "memory": { "total_bytes": 16777216000, "used_bytes": 8388608000, "free_bytes": 8388608000 }
}
```

- `cpu.percent`: `float64`, porcentaje `0-100`. Se calcula como `(1 - idleDelta/totalDelta) * 100` leyendo `/proc/stat` dos veces con `500ms` entre lecturas.
- `memory.total_bytes/used_bytes/free_bytes`: `uint64`, **bytes** (el agente lee KiB de `/proc/meminfo` y multiplica por `1024`).
  - `total_bytes` = `MemTotal`.
  - `free_bytes` (disponible) = `MemAvailable` (con fallback a `MemFree` si no existe).
  - `used_bytes` = `total - disponible`.

Este contrato debe coincidir con `contracts/report.md`: categoría → subcategoría → `number | bool`.

## Logs

- Cada envío deja una línea `Sending metrics request payload=...`.
- Los fallos de red, de construcción del request o de código HTTP no-2xx dejan `Algo a fallado` en el log (sin detalle). Para depurar, revisa conectividad, `baseUrl` y que el endpoint acepte el contrato.

## Errores comunes

| Síntoma | Causa probable |
|---|---|
| `no se encontro config.json` | No hay `config.json` en el directorio de trabajo ni junto al binario. |
| `este agente solo soporta Linux por ahora` | Lo ejecutaste en macOS/Windows. Solo corre en Linux. |
| `no se pudo leer /proc/stat` o `/proc/meminfo` | Contenedor/entorno sin `/proc` montado. |
| Sin logs de envío tras `Started` | Revisa `intervalSeconds` y que `Collect()` no esté fallando en silencio (el `send` retorna sin log si `Collect()` o `json.Marshal` fallan). |
