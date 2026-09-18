# TODO

## 1. Alinear el contrato de metricas

- [x] Definir el JSON final que enviara el agente.
- [x] Verificar que el agente envia CPU y memoria sin temperatura.
- [x] Crear el mismo contrato en la API.
- [x] Añadir la validacion del payload en la API.

## 2. Crear el registro de servidores

- [x] Definir el modelo minimo de un servidor.
- [x] Implementar `POST /api/v1/servers`.
- [x] Implementar `GET /api/v1/servers`.
- [x] Implementar `GET /api/v1/servers/:id`.
- [x] Elegir almacenamiento temporal en memoria.

## 3. Crear la identidad basica por servidor

- [ ] Generar una credencial al registrar un servidor.
- [ ] Asociar la credencial con un `serverId`.
- [ ] Mantener la validacion dentro de un modulo de autenticacion aislado.
- [ ] No registrar credenciales completas en los logs.

## 4. Guardar las metricas recibidas

- [ ] Implementar `POST /api/v1/servers/:id/metrics`.
- [ ] Validar la credencial del servidor.
- [ ] Validar el payload de metricas.
- [ ] Guardar cada reporte asociado a su servidor.
- [ ] Asignar `recordedAt` desde la API.

## 5. Actualizar `lastSeenAt` y el estado

- [ ] Actualizar `lastSeenAt` al recibir metricas.
- [ ] Usar `unreported` cuando nunca se haya recibido un reporte.
- [ ] Usar `online` dentro del intervalo esperado.
- [ ] Usar `offline` cuando se supere el intervalo esperado.

## 6. Implementar el publicador SSE

- [ ] Crear un publicador de eventos por servidor.
- [ ] Publicar una metrica cada vez que se reciba un reporte.
- [ ] Implementar `GET /api/v1/servers/:id/metrics/stream`.
- [ ] Limpiar conexiones SSE cerradas.
- [ ] Añadir heartbeats para mantener las conexiones activas.

## 7. Añadir un dashboard minimo

- [ ] Mostrar la lista de servidores.
- [ ] Mostrar estado y ultimo reporte.
- [ ] Conectarse al stream SSE del servidor seleccionado.
- [ ] Mostrar CPU y memoria en tiempo real.
