-- OctoOps schema (dev.md sección 4)
-- Aplicado automáticamente por docker-entrypoint-initdb.d la primera vez

CREATE TABLE users (
  id             uuid PRIMARY KEY,
  github_user_id text UNIQUE NOT NULL,
  username       text NOT NULL,
  email          text,
  avatar_url     text,
  created_at     timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE servers (
  id         uuid PRIMARY KEY,
  user_id    uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  name       text NOT NULL,
  public_key text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX ON servers(user_id);

CREATE TABLE metrics (
  id          bigserial PRIMARY KEY,
  server_id   uuid NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
  metrics     jsonb NOT NULL,
  recorded_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX ON metrics(server_id, recorded_at DESC);

CREATE TABLE refresh_tokens (
  id         uuid PRIMARY KEY,
  user_id    uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  expires_at timestamptz NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX ON refresh_tokens(user_id);
