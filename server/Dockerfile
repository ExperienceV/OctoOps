# syntax=docker/dockerfile:1

# ── Etapa 1: compilar el agente Go (binario estático) ─────────────────────────
FROM golang:1.25-alpine AS agent-builder
WORKDIR /src
COPY agent/ ./agent/
WORKDIR /src/agent
RUN CGO_ENABLED=0 go build -o /out/octo-agent .

# ── Etapa 2: compilar el backend Rust ─────────────────────────────────────────
FROM rust:1-bookworm AS server-builder
RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential cmake clang pkg-config \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /src
COPY server/ ./server/
WORKDIR /src/server
RUN cargo build --release

# ── Etapa 3: imagen de runtime ────────────────────────────────────────────────
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home app

WORKDIR /app
COPY --from=server-builder /src/server/target/release/octoops-backend /app/octoops-backend
COPY --from=agent-builder /out/octo-agent /app/agent/octo-agent

# El binario del agente se sirve desde dentro de la imagen.
ENV AGENT_BINARY_PATH=/app/agent/octo-agent

USER app
EXPOSE 8000
CMD ["/app/octoops-backend"]
