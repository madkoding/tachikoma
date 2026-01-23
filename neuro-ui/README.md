# Neuro UI

Interfaz de usuario para el sistema Neuro - disponible como **aplicación web** y **aplicación de escritorio** (Windows, Linux, macOS).

## 🚀 Inicio Rápido

### Desarrollo Web
```bash
npm install
npm run dev
```
Servidor en `http://localhost:5173`

### Desarrollo Desktop
```bash
npm run tauri:dev
```
Abre la aplicación de escritorio con hot-reload.

## 🖥️ Aplicación de Escritorio

NEURO-OS puede compilarse como aplicación de escritorio nativa usando **Tauri**.

### Build para tu plataforma actual
```bash
npm run tauri:build
```

### Documentación completa
- [QUICKSTART_DESKTOP.md](QUICKSTART_DESKTOP.md) - Inicio rápido
- [DESKTOP_BUILD.md](DESKTOP_BUILD.md) - Guía completa de compilación
- Ver también: [../NEURO_DESKTOP_SETUP.md](../NEURO_DESKTOP_SETUP.md)

## ⚙️ Configuración

### Variables de Entorno

Copia `.env.example` a `.env` y configura las variables necesarias:

```bash
cp .env.example .env
```

#### Variables Disponibles

- `VITE_API_URL`: URL del backend de Neuro
  - **Desarrollo local**: Dejar vacío para usar el proxy de Vite (`/api` → `http://0.0.0.0:3000`)
  - **Producción/Despliegue**: URL completa del backend (ej: `https://api.tudominio.com/api`)

### Desarrollo Local

```bash
npm install
npm run dev
```

El servidor de desarrollo estará disponible en `http://localhost:5173`

### Producción

Para desplegar en un dominio diferente al del backend:

1. Configura `VITE_API_URL` con la URL del backend:
   ```
   VITE_API_URL=https://api.tudominio.com/api
   ```

2. Construye la aplicación:
   ```bash
   npm run build
   ```

3. Despliega el contenido de la carpeta `dist`

## CORS

Asegúrate de que el backend tenga configurado CORS para permitir peticiones desde el dominio donde se aloja el UI.
