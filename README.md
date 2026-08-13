# Propósito del sistema

## ¿Qué problema resuelve?

Hoy en día, monitorizar infraestructura y monitorizar dispositivos físicos/IoT son dos mundos que casi nunca se hablan. Existen herramientas maduras para lo primero (Zabbix, Netdata, Grafana + Prometheus) y plataformas separadas para lo segundo (Home Assistant, ThingsBoard, plataformas cloud de fabricantes de sensores). Si alguien administra servidores Linux **y** además tiene proyectos con microcontroladores (una incubadora, un invernadero, un tanque de agua, un taller), termina usando dos sistemas distintos, con dos logins, dos formas de configurar alertas, y ningún panorama unificado.

Este sistema nace para cubrir ese hueco: **un solo lugar donde ver "todo lo que reporta datos"**, sin importar si es un servidor Linux con systemd o un ESP32 con un sensor de temperatura soldado a mano.

## ¿Para quién es?

Pensado inicialmente para un uso personal/pequeña escala (1-10 nodos) — alguien que:
- Administra sus propios servidores (VPS, home server, proyectos personales) y quiere saber si están sanos sin entrar por SSH a cada rato.
- Tiene proyectos paralelos con microcontroladores donde algo físico necesita ser monitoreado remotamente — una incubadora de huevos, un cultivo, un cuarto de equipos, un sistema de riego.

No está diseñado (por ahora) para operar a la escala de una empresa con cientos de servidores ni miles de sensores — esa restricción es intencional: mantiene el sistema simple, barato de correr, y fácil de entender de punta a punta.

## La idea central: "Módulos" como concepto abierto

La parte más importante del diseño no es el monitoreo de servidores (eso ya existe en muchas herramientas) — es la sección **Modules**. La apuesta es que **cualquier cosa que pueda mandar un JSON por HTTP puede convertirse en algo monitoreado**, sin tener que tocar el backend cada vez que se conecta un dispositivo nuevo.

Ejemplo concreto: hoy es una incubadora reportando temperatura, humedad y consumo energético cada 24 horas. Mañana podría ser:
- Un sensor de nivel de agua en un tinaco.
- Un contador de visitas en un espacio físico.
- Un sistema de riego reportando humedad de tierra.
- Cualquier proyecto personal con un ESP32/Arduino que hoy no existe todavía.

El sistema no necesita "saber" de antemano qué es una incubadora o qué mide un sensor de agua — cada conector define su propio conjunto de métricas al registrarse, y el sistema se adapta.

## Qué valor entrega

1. **Visibilidad centralizada** — un solo dashboard para saber si un servidor está caído o si la incubadora se está sobrecalentando, sin cambiar de herramienta.
2. **Alertas tempranas** — enterarse de un problema (servidor sin responder, temperatura fuera de rango) antes de que se convierta en una falla real o una pérdida (ej. huevos perdidos por una falla térmica).
3. **Extensibilidad sin fricción** — agregar un nuevo tipo de sensor/dispositivo no requiere cambios de código en el backend, solo definir su esquema de datos.
4. **Control total de los datos** — al ser un sistema propio y autoalojado (self-hosted), no depende de servicios cloud de terceros ni de sus límites, precios o políticas de privacidad.

## Qué NO busca ser

- No busca competir con herramientas enterprise de observabilidad (Datadog, New Relic) — no hay tracing distribuido, ni APM, ni análisis de logs a gran escala.
- No busca ser una plataforma IoT genérica con miles de integraciones prearmadas (como Home Assistant) — el enfoque es simplicidad y control total del propio esquema de datos, no compatibilidad universal con dispositivos de terceros.
- No está pensado (todavía) para multiusuario a gran escala ni para venderse como SaaS — es una herramienta personal que puede crecer si el caso de uso lo justifica.