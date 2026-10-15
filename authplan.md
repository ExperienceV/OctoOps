# Plan de auth (GitHub + JWT)

Autenticación con GitHub OAuth y JWTs de access/refresh en cookies httpOnly.

## Tokens

| Token | Duración | Cookie |
|---|---|---|
| Access | 30 min (`1800s`) | `access_token` |
| Refresh | 7 días (`604800s`) | `refresh_token` |

Cookies: `httpOnly`, `SameSite=Lax`, `Path=/` (`Secure` en producción).

## Rutas

| Método | Ruta | Qué hace |
|---|---|---|
| `GET` | `/auth/github/login` | Redirige a GitHub para autorizar (no requiere nada). |
| `GET` | `/auth/github/callback` | Recibe el `code` de GitHub, lo intercambia, trae el perfil, crea/actualiza el `User`, emite `access`+`refresh`, setea cookies y redirige al front. |
| `POST` | `/auth/refresh` | Lee la cookie `refresh_token`, la rota (revoca + emite nueva), setea cookies nuevas. |
| `GET` | `/auth/me` | Lee la cookie `access_token`, valida y devuelve el usuario actual. |
| `POST` | `/auth/logout` | Revoca el `refresh_token` y limpia las cookies. |

## Flujo

```
login → (GitHub autoriza) → callback → setea cookies → front
        └─ access expira (30min) → /auth/refresh → cookies nuevas
        └─ logout → revoca refresh + borra cookies
```

## Dependencias por ruta

- `login` → `Config` (+ `url` para armar la URL de autorización).
- `callback` → `github` + `jwt` + `db` (upsert user, insert refresh) + `cookies`.
- `refresh` → `jwt` + `db` (revocar/insertar refresh) + `cookies`.
- `me` → `cookies` + `jwt` + `db` (get user).
- `logout` → `cookies` + `jwt` + `db` (revocar refresh).
