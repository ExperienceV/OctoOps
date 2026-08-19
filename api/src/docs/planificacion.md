Sistema para gestionar y monitorizar servidores linux además de proyectos IoT y sistemas embebidos.

### Servidor

# PREGUNTAS

## ¿Cómo logro obtener las métricas de un servidor linux?
Instalar un agente en cada servidor el cual se encargará de enviar las métricas
a un servidor central en un intervalo de tiempo determinado. Un agénte es un proceso
que se ejecuta en segundo plano y recopila información del sistema. El cuál se desarrollará
por preferencia en un lenguaje compilado para tener mayor eficiencia y menor uso de recursos.

--- REQUIERE API KEY ESTATICA ---

## ¿Cómo logro obtener las métricas de un dispositivo IoT?
El microcontrolador con acceso a internet deberá enviar las métricas en formato JSON
por medio del protocolo HTTPS a una API REST que se encargará de almacenar las métricas
en una base de datos. La API debe aceptar JSONs con una estructura fuertemente tipáda y dinámica.

--- REQUIERE API KEY ESTATICA ---


## ¿Qué metricas son importantes para un servidor linux?
Por ahora solo planeo enforcarme en CPU %, RAM %, DISK %, RED Incoming/Outcoming.


## ¿Qué metricas son importantes para un dispositivo IoT?
Las métricas para cualquier dispositivo IoT principalmente será RSSI, Voltage/Bateria en caso
de no estar conectado a la red eléctrica, memoria ram disponible, Uptime/Reset reason, y para
las métricas de los distintos sensores ya es más dinámico para así poder implementar distintos
sensores. Pero deben cumplir con una estructura base para que el sistema pueda entenderlas y
gestionarlas.


# SEGURIDAD MINIMA EN IoT y SERVIDORES

El dispositivo IoT debe tener implementado un sistema de autenticación
para que el servidor pueda identificar al dispositivo. Por ahora contará con una API_KEY
estática generada por el servidor.

El agente del servidor también debe tener implementado un sistema de autenticación
para que el servidor pueda identificar al agente.
