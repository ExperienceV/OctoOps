# Despliegue a producción

El backend se empaqueta en una imagen Docker (`ghcr.io/experiencev/octoops`) que
**incluye el binario del agente** (`/app/agent/octo-agent`). En cada push a `main`,
GitHub Actions construye la imagen, la sube a GHCR y se conecta por SSH al VPS para
desplegarla.

## Arquitectura

```
Internet ──▶ Caddy (443, HTTPS) ──▶ server:8000 (backend) ──▶ db:5432 (Postgres)
```

- **Caddy** termina TLS (Let's Encrypt) y hace de reverse proxy.
- **server** sirve la API, el instalador (`/servers/install/…`) y el binario del agente.
- **db** es Postgres, sin puerto expuesto al host.

## 1. Setup único del VPS (EC2)

### Instalar Docker

```bash
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER
# cierra sesión y vuelve a entrar (o: newgrp docker)
```

### Crear `/opt/octoops` con los archivos

```bash
sudo mkdir -p /opt/octoops && cd /opt/octoops

# traer los archivos desde el repo (público)
git clone --depth 1 https://github.com/ExperienceV/OctoOps.git /tmp/octoops
cp /tmp/octoops/docker-compose.prod.yml /opt/octoops/
cp /tmp/octoops/Caddyfile               /opt/octoops/
cp /tmp/octoops/server/schema.sql       /opt/octoops/
```

### Crear el `.env` (secretos)

```bash
cd /opt/octoops
cat > .env <<'EOF'
POSTGRES_PASSWORD=<openssl rand -hex 16>
JWT_SECRET=<openssl rand -hex 32>
GITHUB_CLIENT_ID=<tu OAuth App client id>
GITHUB_CLIENT_SECRET=<tu OAuth App client secret>
EOF
```

> `GITHUB_CLIENT_ID/SECRET` son los mismos de tu OAuth App de GitHub. Además debes
> agregar el callback de producción a esa app: `https://octoops.apeiriumx.com/auth/github/callback`.

### Acceso a GHCR (para `pull`)

Si el paquete de imagen es **privado** (default de GHCR), el VPS debe autenticarse:

```bash
echo <PAT con scope read:packages> | docker login ghcr.io -u <tu-usuario-github> --password-stdin
```

O más simple: haz el paquete **público** en GitHub → `Package settings → Change visibility → Public`.

## 2. Secrets de GitHub

En el repo → **Settings → Secrets and variables → Actions → New repository secret**:

| Secret | Valor |
|---|---|
| `VPS_HOST` | IP o dominio del EC2 |
| `VPS_USER` | usuario SSH (ej. `ubuntu`) |
| `VPS_SSH_KEY` | clave privada SSH (para conectarse al VPS) |

El token de GHCR usa `GITHUB_TOKEN` automáticamente, no hay que crear secret extra.

## 3. DNS

Apuntar `octoops.apeiriumx.com` (registro `A`) a la IP pública del EC2.
**Hasta que el DNS no resuelva, Caddy no puede emitir el certificado** — el
contenedor arranca igual y Caddy reintenta automáticamente.

## 4. Cómo se despliega

1. Haces push a `main`.
2. GitHub Actions construye la imagen (backend + binario del agente) y la sube a GHCR.
3. Se conecta por SSH al VPS y ejecuta:
   ```bash
   cd /opt/octoops
   docker compose -f docker-compose.prod.yml pull
   docker compose -f docker-compose.prod.yml up -d
   ```

## Primer arranque manual (opcional)

Para levantar la primera vez sin esperar a un push:

```bash
cd /opt/octoops
docker compose -f docker-compose.prod.yml pull
docker compose -f docker-compose.prod.yml up -d
```

## Verificar

```bash
docker compose -f docker-compose.prod.yml ps
docker compose -f docker-compose.prod.yml logs -f server
curl -I https://octoops.apeiriumx.com/auth/github/login
```
