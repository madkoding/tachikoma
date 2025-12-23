# Neuro UI

Interfaz de usuario para el sistema Neuro.

## Configuración

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
