# Router `auth`

Autenticación del usuario con GitHub OAuth + JWT (access/refresh) en cookies httpOnly.

## Tokens

| Token | Duración | Cookie |
|---|---|---|
| Access | 30 min | `access_token` |
| Refresh | 7 días | `refresh_token` |

## Endpoints

| Método | Ruta | Descripción |
|---|---|---|
| `GET` | `/auth/github/login` | Redirige a GitHub. |
| `GET` | `/auth/github/callback` | Completa el login y setea cookies. |
| `POST` | `/auth/refresh` | Rota el refresh y emite tokens nuevos. |
| `GET` | `/auth/me` | Devuelve el usuario actual. |
| `POST` | `/auth/logout` | Revoca el refresh y limpia cookies. |

## Detalle

### GET /auth/github/login

Redirige a `https://github.com/login/oauth/authorize` con `client_id`, `redirect_uri` y `scope=read:user user:email`.

### GET /auth/github/callback?code=...

1. Cambia `code` por un access token de GitHub.
2. Trae el perfil y hace upsert del usuario.
3. Emite `access` + `refresh` y guarda el refresh en la DB.
4. Setea cookies y redirige al front (`FRONTEND_URL`).

Errores: `401` si falla el intercambio/consulta con GitHub, `500` si falla la DB.

### POST /auth/refresh

Lee la cookie `refresh_token`, la valida, la **revoca** y emite un par nuevo (rotación de un solo uso).

Respuesta: `204` con cookies nuevas. `401` si el refresh es inválido o vencido.

### GET /auth/me

Lee la cookie `access_token`, la valida y devuelve el usuario actual.

Respuesta `200`:

```json
{ "id": "uuid", "username": "octocat", "email": "a@b.com", "avatar_url": "https://..." }
```

`401` si no hay access válido.

### POST /auth/logout

Revoca el `refresh_token` y limpia ambas cookies. Respuesta `204`.

## Cookies

- `httpOnly`, `SameSite=Lax`, `Path=/`.
- `Secure` en producción.
