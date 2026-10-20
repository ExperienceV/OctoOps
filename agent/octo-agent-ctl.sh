#!/bin/sh
# octo-agent-ctl — controla el agente OctoOps en segundo plano.
# Uso: octo-agent-ctl {start|stop|restart|status}
#
# Vive en el mismo directorio que el binario `octo-agent`.
# Usa nohup + archivo de PID para mantenerlo corriendo en segundo plano.

DIR="$(cd "$(dirname "$0")" && pwd)"
BIN="$DIR/octo-agent"
PID="$DIR/octo-agent.pid"
LOG="$DIR/octo-agent.log"

is_running() {
  [ -f "$PID" ] && kill -0 "$(cat "$PID")" 2>/dev/null
}

start() {
  if is_running; then
    echo "ya está corriendo (pid $(cat "$PID"))"
    return 0
  fi
  nohup "$BIN" >>"$LOG" 2>&1 &
  echo $! > "$PID"
  echo "iniciado (pid $(cat "$PID"))"
}

stop() {
  if is_running; then
    kill "$(cat "$PID")" 2>/dev/null
    rm -f "$PID"
    echo "detenido"
  else
    rm -f "$PID"
    echo "no estaba corriendo"
  fi
}

restart() {
  stop
  sleep 1
  start
}

status() {
  if is_running; then
    echo "corriendo (pid $(cat "$PID"))"
  else
    echo "detenido"
  fi
}

case "${1:-}" in
  start)   start ;;
  stop)    stop ;;
  restart) restart ;;
  status)  status ;;
  *) echo "uso: $0 {start|stop|restart|status}"; exit 1 ;;
esac
