# OctoOps

A **self-hosted** platform to monitor Linux servers and IoT devices from a single place.
A lightweight **agent** (Go) is installed on each node, collects metrics and reports them
—signed with **Ed25519**— to a **backend** (Rust + PostgreSQL). The **web client** renders
them in a dashboard.

---

## Architecture

```
                 ┌──────────────┐
   user ───────▶ │  Web client  │ ──HTTPS──▶ ┌─────────┐      ┌──────────┐
                 └──────────────┘            │  Caddy  │ ───▶ │ Backend  │──▶ Postgres
                                             │ (proxy) │      │  (Rust)  │
                                             └─────────┘      └──────────┘
                                                   ▲
                                                   │ signed report (Ed25519)
                                             ┌─────────┐
                                             │  Agent  │  (Go · on each node)
                                             └─────────┘
```

- **Caddy** terminates TLS (Let's Encrypt), serves the client's static files and reverse
  proxies to the backend.
- **Backend** (Rust) exposes the API and the installer, and persists metrics in PostgreSQL.
- **Agent** (Go) runs on the monitored nodes and reports every N seconds.

## Components

| Component | Language | Description |
|---|---|---|
| `server/` | Rust (axum, sqlx) | REST API, auth, installation, metric persistence. |
| `agent/` | Go (stdlib only) | Collects CPU/memory and reports signed. |
| Web client | static (HTML/JS) | Dashboard (separate repo in production, served by Caddy). |
| `Caddyfile` | — | Reverse proxy + automatic HTTPS. |
| `docker-compose.prod.yml` | — | Production stack: `db` + `server` + `caddy`. |

## Tech stack

- **Backend**: Rust, axum 0.8, sqlx 0.8, PostgreSQL 17, `ed25519-dalek`.
- **Auth**: GitHub OAuth + JWT (`access` 30 min / `refresh` 7 days) in `httpOnly` cookies.
- **Agent**: Go 1.25, no external dependencies, Ed25519 from stdlib.
- **Deploy**: Docker + Caddy (Let's Encrypt) + GitHub Actions (CI/CD → GHCR).

## Repo layout

```
.
├── server/                # Backend (Rust)
│   ├── src/
│   │   ├── auth/          # GitHub OAuth, JWT, cookies
│   │   ├── db/            # connection + queries
│   │   ├── routes/        # auth, servers, report
│   │   ├── types/         # DTOs
│   │   ├── enrollment.rs  # enrollment token (SHA-256 hash)
│   │   ├── state.rs       # AppState (pool + config)
│   │   └── main.rs
│   ├── schema.sql         # DB schema
│   ├── Dockerfile         # image (backend + agent binary)
│   └── Cargo.toml
├── agent/                 # Agent (Go)
│   ├── internal/
│   │   ├── collector/     # system metrics (CPU, memory)
│   │   ├── config/        # config.json read/write
│   │   ├── enroll/        # registration + Ed25519 keypair
│   │   └── reporter/      # signed report delivery
│   ├── types/             # Report contract
│   ├── main.go
│   └── octo-agent-ctl.sh  # start/stop/restart/status
├── contracts/             # data contracts
│   └── report.md
├── docs/                  # documentation
│   ├── guide.md           # full guide (flows, endpoints, responses)
│   ├── deploy.md          # production deployment
│   ├── auth.md · servers.md · report.md
├── Caddyfile              # reverse proxy + HTTPS
├── docker-compose.yml     # DB only (dev)
├── docker-compose.prod.yml
└── .github/workflows/     # CI/CD
```

## How it works

1. **Login**: GitHub OAuth → JWT in `httpOnly` cookies.
2. **Create server**: `POST /servers` → **enrollment token** (random, SHA-256 hash stored,
   single-use, 24 h).
3. **Install agent**: `curl …/servers/install/<token> | sh` → downloads binary + `config.json`
   + control script.
4. **Register**: the agent generates an Ed25519 keypair and sends its `public_key`; it becomes
   `active`.
5. **Report**: every N seconds it signs `timestamp + "\n" + body` and posts to
   `POST /servers/agents/report` (headers `X-Agent-ID`, `X-Timestamp`, `X-Signature`).
6. **Read**: the client queries `GET /servers`, `GET /servers/:id`, `GET /servers/:id/metrics`.

> Full details in [`docs/guide.md`](docs/guide.md).

## Endpoints (summary)

| Method | Route | Auth | Description |
|---|---|---|---|
| `GET` | `/auth/github/login` | public | GitHub login. |
| `GET` | `/auth/me` | JWT | Current user. |
| `POST` | `/auth/logout` | JWT | Log out. |
| `GET` | `/servers` | JWT | List servers (with status). |
| `POST` | `/servers` | JWT | Create server → install command. |
| `GET` | `/servers/:id` | JWT | Server detail. |
| `GET` | `/servers/:id/metrics` | JWT | Metric history. |
| `GET` | `/servers/install/bin` | public | Agent binary. |
| `GET` | `/servers/install/:token` | token | Install script. |
| `POST` | `/servers/agents/register` | token | Register agent. |
| `POST` | `/servers/agents/report` | Ed25519 | Report metrics. |

## Local development

```bash
# 1. DB
docker compose up -d

# 2. Backend
cd server
cp .env.example .env   # fill GITHUB_CLIENT_ID/SECRET and JWT_SECRET
cargo run              # listens on 127.0.0.1:8000

# 3. Web client (static files + proxy to the backend)
#    see docs/deploy.md or serve the statics with a proxy to :8000
```

## Deployment

See [`docs/deploy.md`](docs/deploy.md): Docker + Caddy + GitHub Actions (push to `main` →
build → GHCR → SSH deploy).

## Contracts

- [`contracts/report.md`](contracts/report.md) — metrics payload:
  `{ "category": { "subcategory": number | bool } }`.
- [`contracts/server.md`](contracts/server.md) — server/device node shape (`kind`, `status`, …).
- [`docs/guide.md`](docs/guide.md) — server, user and metric contracts, statuses and response
  codes.
