# TACHIKOMA-AGENT

Microservicio de herramientas de agente para TACHIKOMA-OS.

## Descripción

Este servicio proporciona capacidades de agente IA:
- **Búsqueda web** vía Searxng (metabuscador privado)
- **Ejecución de comandos** con restricciones de seguridad

## Endpoints

### Health Check
```
GET /api/health
```

### Web Search
```
POST /api/agent/search
Content-Type: application/json

{
  "query": "rust programming language",
  "categories": ["general"],
  "engines": ["google", "duckduckgo"],
  "language": "es",
  "page": 1,
  "time_range": "month",
  "max_results": 10
}
```

**Respuesta:**
```json
{
  "success": true,
  "results": [
    {
      "title": "Rust Programming Language",
      "url": "https://rust-lang.org",
      "content": "A systems programming language...",
      "engine": "google",
      "score": 0.95
    }
  ],
  "total": 100,
  "suggestions": ["rust tutorial", "rust book"],
  "answers": []
}
```

### Command Execution
```
POST /api/agent/execute
Content-Type: application/json

{
  "command": "ls",
  "args": ["-la", "/home"],
  "working_dir": "/tmp",
  "timeout_secs": 30
}
```

**Respuesta:**
```json
{
  "success": true,
  "exit_code": 0,
  "stdout": "total 4\ndrwxr-xr-x 2 user user 4096 ...",
  "stderr": "",
  "truncated": false
}
```

### List Allowed Commands
```
GET /api/agent/commands
```

**Respuesta:**
```json
{
  "commands": ["ls", "cat", "head", "tail", "wc", "grep", "find", "which", "date", "cal", "uptime", "whoami", "pwd", "echo", "df", "du"]
}
```

## Seguridad

### Comandos Bloqueados
Los siguientes comandos están bloqueados por seguridad:
- `rm`, `rmdir`, `dd`, `mkfs`, `fdisk`, `format`
- `shutdown`, `reboot`, `poweroff`, `halt`, `init`
- `kill`, `killall`, `pkill`
- `chmod`, `chown`, `chgrp`
- `sudo`, `su`, `doas`
- `wget`, `curl`, `nc`, `netcat`
- `ssh`, `scp`, `rsync`
- `mount`, `umount`
- `iptables`, `systemctl`, `service`
- `docker`, `podman`
- `eval`, `exec`

### Patrones Peligrosos
Los argumentos no pueden contener:
- Pipes y encadenamiento: `|`, `;`, `&&`, `||`
- Sustitución de comandos: `` ` ``, `$(`, `${`
- Redirección: `>`, `>>`, `<`, `<<`
- Rutas peligrosas: `/dev/`, `/proc/`, `/sys/`, `/root`
- Navegación padre: `..`

## Variables de Entorno

| Variable | Descripción | Default |
|----------|-------------|---------|
| `HOST` | Host de escucha | `0.0.0.0` |
| `PORT` | Puerto de escucha | `3005` |
| `SEARXNG_URL` | URL del servidor Searxng | `http://localhost:8080` |
| `ALLOWED_COMMANDS` | Lista de comandos permitidos (separados por coma) | ver código |
| `RUST_LOG` | Nivel de logging | `info` |

## Desarrollo

### Ejecutar localmente
```bash
cd tachikoma-agent
SEARXNG_URL=http://localhost:8080 cargo run
```

### Build Docker (desarrollo)
```bash
docker build -f Dockerfile.dev -t tachikoma-agent:dev .
docker run -p 3005:3005 -e SEARXNG_URL=http://host.docker.internal:8080 tachikoma-agent:dev
```

### Build Docker (producción)
```bash
docker build -t tachikoma-agent:latest .
```

## Arquitectura

```
tachikoma-agent/
├── src/
│   ├── main.rs         # Entry point
│   ├── config.rs       # Configuración
│   ├── routes.rs       # Rutas HTTP
│   ├── handlers.rs     # Handlers de request
│   ├── searxng.rs      # Cliente de Searxng
│   └── executor.rs     # Ejecutor de comandos
├── Dockerfile          # Build producción
├── Dockerfile.dev      # Build desarrollo
└── README.md
```

## Integración

Este servicio se integra con:
- **Searxng**: Para búsquedas web privadas
- **tachikoma-backend**: Como gateway API
- **tachikoma-chat**: Para proporcionar herramientas al LLM
