# 🔧 Solución de Problemas de Acceso Remoto

## Problema Identificado

Cuando se accede a NEURO-OS desde una máquina remota, las UIs (User y Admin) no pueden comunicarse correctamente con el backend, incluso cuando todos los servicios están corriendo.

## Causa Raíz

1. **Proxy de Vite mal configurado**: Los proxies apuntaban a `http://0.0.0.0:3000` en lugar de `http://localhost:3000`
2. **Procesos huérfanos**: El script `stop.sh` no mataba todos los procesos correctamente, dejando instancias duplicadas
3. **Puertos bloqueados**: Los puertos no se liberaban completamente entre reinicios

## Solución Implementada

### 1. Configuración del Proxy (Vite)

**Archivos modificados:**
- `neuro-ui/vite.config.ts`
- `neuro-admin/vite.config.ts`

**Cambios:**
```typescript
// ANTES (❌ No funciona bien remotamente)
proxy: {
  '/api': {
    target: 'http://0.0.0.0:3000',
    changeOrigin: true,
  }
}

// DESPUÉS (✅ Funciona correctamente)
proxy: {
  '/api': {
    target: 'http://localhost:3000',
    changeOrigin: true,
    secure: false,
    ws: true,  // Soporte WebSocket
  }
}
```

### 2. Script de Detención Mejorado

**Archivo modificado:** `stop.sh`

**Mejoras:**
- Usa `pkill -9` para matar procesos más agresivamente
- Libera puertos específicos con `fuser -k`
- Mata procesos más específicos (evita matar otros vite)
- Espera 1 segundo después de liberar puertos

```bash
# Matar backend
pkill -9 -f "neuro-backend"
pkill -9 -f "cargo.*run.*neuro-backend"

# Matar frontends específicos
pkill -9 -f "vite.*neuro-ui"
pkill -9 -f "vite.*neuro-admin"

# Liberar puertos
fuser -k 3000/tcp
fuser -k 5173/tcp
fuser -k 5174/tcp
sleep 1
```

### 3. Script de Diagnóstico

**Nuevo archivo:** `diagnose.sh`

Ejecuta `./diagnose.sh` para verificar:
- ✅ Estado de todos los puertos
- ✅ Salud de endpoints HTTP
- ✅ Contenedores Docker
- ✅ Procesos en ejecución
- ✅ URLs de acceso (local y remoto)
- ✅ Configuración del proxy

## Cómo Usar

### Procedimiento de Reinicio Limpio

```bash
# 1. Detener todos los servicios
./stop.sh

# 2. Verificar que todo esté detenido
./diagnose.sh

# 3. Si hay procesos residuales, matarlos manualmente
# (el diagnose.sh te mostrará los PIDs)

# 4. Iniciar servicios
./start.sh

# 5. Verificar que todo funcione
./diagnose.sh
```

### Acceso Local

```bash
# User UI
http://localhost:5173

# Admin UI
http://localhost:5174

# Backend API
http://localhost:3000/api/health
```

### Acceso Remoto

Desde otra máquina en la red local:

```bash
# Usando IP
http://192.168.X.X:5173    # User UI
http://192.168.X.X:5174    # Admin UI

# Usando hostname
http://tachikoma:5173      # User UI
http://tachikoma:5174      # Admin UI
```

**Nota importante:** Las UIs usan proxy, por lo que las peticiones al backend se hacen **internamente** desde el servidor Vite, no desde el navegador del cliente. Esto permite que funcione correctamente en acceso remoto.

## Firewall

Si el acceso remoto no funciona, verifica el firewall:

```bash
# Ver estado
sudo ufw status

# Permitir puertos
sudo ufw allow 3000/tcp
sudo ufw allow 5173/tcp
sudo ufw allow 5174/tcp

# Recargar
sudo ufw reload
```

## Arquitectura de Red

```
┌─────────────────────────────────────────────────────────────┐
│                     Máquina Remota                          │
│                                                             │
│  ┌──────────────┐                                          │
│  │  Navegador   │                                          │
│  │              │                                          │
│  └──────┬───────┘                                          │
│         │                                                   │
│         │ HTTP Request                                     │
│         │ http://tachikoma:5173/api/chat                   │
│         │                                                   │
└─────────┼───────────────────────────────────────────────────┘
          │
          │ Internet/Red Local
          │
┌─────────┼───────────────────────────────────────────────────┐
│         │                Servidor (tachikoma)              │
│         ▼                                                   │
│  ┌─────────────┐                                           │
│  │ Vite Server │ :5173                                     │
│  │  (User UI)  │                                           │
│  └──────┬──────┘                                           │
│         │                                                   │
│         │ Proxy interno                                    │
│         │ /api/* → http://localhost:3000                   │
│         │                                                   │
│         ▼                                                   │
│  ┌─────────────┐                                           │
│  │   Backend   │ :3000                                     │
│  │  (Rust API) │                                           │
│  └─────────────┘                                           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

**Flujo de petición:**
1. Navegador remoto → `http://tachikoma:5173/api/chat`
2. Vite Server recibe la petición
3. Vite proxy redirige internamente → `http://localhost:3000/api/chat`
4. Backend responde a Vite
5. Vite responde al navegador

## Verificación

Después de aplicar los cambios:

```bash
# 1. Reiniciar servicios
./stop.sh && sleep 2 && ./start.sh

# 2. Ejecutar diagnóstico
./diagnose.sh

# 3. Verificar desde máquina remota
curl http://tachikoma:5173
curl http://tachikoma:3000/api/health

# 4. Abrir en navegador remoto
# http://tachikoma:5173
```

## Troubleshooting Adicional

### Problema: "Connection Refused" desde remoto

```bash
# Verificar que el backend escuche en todas las interfaces
netstat -tlnp | grep 3000
# Debería mostrar: 0.0.0.0:3000 o *:3000

# Verificar Vite
netstat -tlnp | grep 5173
# Debería mostrar: 0.0.0.0:5173
```

### Problema: Backend responde pero UI no carga

```bash
# Ver logs del Vite
# Los logs aparecerán en la terminal donde ejecutaste start.sh

# Verificar proxy en tiempo real
tail -f /tmp/vite-*.log
```

### Problema: Procesos duplicados

```bash
# Listar todos los procesos de NEURO-OS
ps aux | grep -E "neuro-backend|vite.*neuro" | grep -v grep

# Matar todos manualmente
pkill -9 -f neuro-backend
pkill -9 -f "vite.*neuro"
fuser -k 3000/tcp 5173/tcp 5174/tcp
```

## Resumen de Archivos Modificados

1. ✅ `neuro-ui/vite.config.ts` - Proxy corregido
2. ✅ `neuro-admin/vite.config.ts` - Proxy corregido
3. ✅ `stop.sh` - Mejorado para matar procesos correctamente
4. ✅ `diagnose.sh` - Nuevo script de diagnóstico (CREADO)
5. ✅ `neuro-ui/.env.development` - Variables de entorno (CREADO)
6. ✅ `neuro-admin/.env.development` - Variables de entorno (CREADO)

## Próximos Pasos

Los cambios ya están aplicados. Para ponerlos en funcionamiento:

```bash
# Aplicar cambios
./stop.sh
sleep 2
./start.sh

# Verificar
./diagnose.sh
```

Ahora deberías poder acceder a NEURO-OS desde cualquier máquina en tu red local usando la IP o hostname del servidor.
