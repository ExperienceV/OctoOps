# Reglas de negocio

Este documento define el comportamiento esperado del sistema — qué está permitido, qué se valida, y qué pasa en cada caso. Sirve como referencia al implementar, independientemente del framework o librería usada.

---

## 1. Usuarios y acceso

- **RN-01**: Todo acceso al dashboard requiere autenticación. No hay vistas públicas.
- **RN-02**: El sistema opera bajo un esquema single-admin al inicio — no hay roles ni permisos diferenciados en la primera versión. Si más adelante se soporta multiusuario, se define como fase posterior (no bloquea el desarrollo actual).
- **RN-03**: Un usuario solo puede ver y administrar los servidores y conectores de los que es `owner_id`.

---

## 2. Servidores

- **RN-04**: Un servidor se crea manualmente desde el dashboard. El sistema genera un `api_key` único al momento de la creación.
- **RN-05**: El `api_key` de un servidor es la única credencial válida para reportar métricas — no hay usuario/contraseña a nivel de agente.
- **RN-06**: Un `api_key` puede ser revocado/regenerado en cualquier momento. Al regenerarse, el agente Go configurado con la key anterior deja de poder reportar hasta que se actualice.
- **RN-07**: Un servidor se considera **online** si su `last_seen_at` es menor o igual a un umbral configurable (por defecto, 90 segundos — el doble del intervalo de reporte esperado de 30-60s). Pasado ese umbral, se considera **offline**.
- **RN-08**: El estado online/offline se calcula al consultar (no se almacena como verdad absoluta), para evitar procesos en segundo plano innecesarios a esta escala.
- **RN-09**: Eliminar un servidor elimina en cascada su historial de métricas (`SERVER_METRICS`) y cualquier regla de alerta asociada.

---

## 3. Conectores (Modules)

- **RN-10**: Un conector se crea manualmente desde el dashboard, igual que un servidor: se genera un `api_key` único y se define su `schema_hint`.
- **RN-11**: El `schema_hint` es responsabilidad del usuario al crear el conector — el sistema no infiere ni valida contra un catálogo de tipos de sensores predefinido. Esto es intencional: el sistema no necesita "conocer" qué es una incubadora.
- **RN-12**: El sistema **no rechaza** un payload que traiga campos fuera del `schema_hint` definido — se guarda igual en el JSONB. El `schema_hint` es una ayuda para la UI y validación de rangos, no un esquema estricto tipo base de datos relacional.
- **RN-13**: Un conector se considera **sin reportar** (offline) si su `last_seen_at` supera un umbral definido por el propio conector (no un valor global — un ESP32 que reporta cada 24h no puede usar el mismo umbral que uno que reporta cada minuto). Por defecto, se sugiere 2x su intervalo esperado de reporte.
- **RN-14**: Eliminar un conector elimina en cascada su historial de lecturas (`CONNECTOR_READINGS`) y cualquier regla de alerta asociada.

---

## 4. Ingesta de datos (servidores y conectores)

- **RN-15**: Toda request de ingesta debe incluir un `X-API-Key` válido correspondiente al `server_id`/`connector_id` de la URL. Si no coincide, se responde 401 y **no se guarda ningún dato**.
- **RN-16**: El timestamp de cada métrica/lectura lo asigna el servidor al momento de recibir la request (`recorded_at` / `received_at`), nunca el dispositivo — evita inconsistencias por relojes no sincronizados en el hardware.
- **RN-17**: No existe deduplicación de lecturas duplicadas — un reintento del dispositivo que produzca el mismo dato dos veces se almacena como dos registros independientes. Esto es aceptable dado el bajo volumen y frecuencia de reporte a esta escala.
- **RN-18**: Cada request exitosa de ingesta actualiza `last_seen_at` de la entidad correspondiente (servidor o conector), independientemente de si los valores reportados están dentro de rangos normales o no.
- **RN-19**: El tamaño máximo de un payload de ingesta está limitado (a definir un valor concreto en implementación) para evitar abuso del endpoint.

---

## 5. Alertas (estructura definida, motor pendiente de fase posterior)

- **RN-20**: Una regla de alerta (`ALERT_RULES`) puede apuntar a un servidor o a un conector (`target_type` + `target_id`), nunca a ambos a la vez.
- **RN-21**: Una regla de alerta se define sobre un único campo (`metric_field`) — para servidores es una columna fija (ej. `cpu_percent`); para conectores es una ruta dentro del JSON del payload (ej. `temperatura`).
- **RN-22**: Crear/editar/eliminar reglas de alerta no requiere que el motor de evaluación esté activo — el CRUD de reglas es independiente de si algo las está evaluando en segundo plano.
- **RN-23**: Mientras el motor de evaluación no esté implementado, ninguna regla genera eventos (`ALERT_EVENTS`) ni notificaciones — su creación es solo preparación para la fase siguiente.
- **RN-24** *(pendiente de definir en fase posterior)*: criterio de resolución de una alerta activa (¿se resuelve sola cuando el valor vuelve a rango normal, o requiere confirmación manual?).

---

## 6. Retención de datos

- **RN-25**: Las métricas de servidores (`SERVER_METRICS`) y las lecturas de conectores (`CONNECTOR_READINGS`) se conservan como máximo **365 días** desde su fecha de registro.
- **RN-26**: Pasado ese periodo, los registros se eliminan de forma definitiva — no hay papelera ni recuperación posterior.
- **RN-27** *(pendiente de definir)*: si se conserva algún dato agregado (promedios diarios/semanales) más allá de los 90 días, o si el histórico se pierde por completo pasado ese plazo.
- **RN-28**: La metadata de servidores y conectores (`SERVERS`, `CONNECTORS`) no está sujeta a esta política — se conserva mientras la entidad exista, independientemente de la antigüedad de sus métricas.

---

## 7. Seguridad y transporte

- **RN-29**: Toda comunicación de ingesta (servidores y conectores) debe ocurrir sobre HTTPS — sin excepción, dado que los dispositivos pueden reportar desde fuera de la red donde vive el servidor.
- **RN-30**: El puerto de la base de datos nunca debe estar expuesto a internet — solo accesible dentro de la red interna del deployment (Docker).
- **RN-31**: Las credenciales (api keys de dispositivos, credenciales de base de datos) se gestionan como variables de entorno — no se hardcodean en el código ni se versionan en el repositorio.

---

## 8. Versionado de API

- **RN-32**: Todos los endpoints de ingesta y del dashboard viven bajo el prefijo `/api/v1/`.
- **RN-33**: Un cambio incompatible en el formato de un endpoint existente no debe romper agentes/dispositivos ya desplegados — implica crear una nueva versión (`/api/v2/`) en paralelo, manteniendo `v1` activo hasta que todos los dispositivos se actualicen.