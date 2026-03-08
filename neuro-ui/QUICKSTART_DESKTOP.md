# 🖥️ Quick Start - TACHIKOMA-OS Desktop

## Desarrollo Local (Hot-Reload)

```bash
cd tachikoma-ui
npm run tauri:dev
```

Esto abrirá la aplicación de escritorio con recarga automática.

## Build de Producción

### Linux (actual sistema)
```bash
cd tachikoma-ui
npm run tauri:build
```

Salida en: `src-tauri/target/release/bundle/`
- `.deb` - Instalador Debian/Ubuntu
- `.AppImage` - Ejecutable portable

### Windows
Compilar en Windows nativo:
```bash
npm run tauri:build:windows
```

### macOS
Compilar en macOS nativo:
```bash
npm run tauri:build:mac
```

## 📖 Documentación Completa

- [TACHIKOMA_DESKTOP_SETUP.md](TACHIKOMA_DESKTOP_SETUP.md) - Resumen completo
- [tachikoma-ui/DESKTOP_BUILD.md](tachikoma-ui/DESKTOP_BUILD.md) - Guía detallada

## ⚠️ Importante

- Primera compilación es lenta (~5-10 min) - dependencias Rust
- Compilaciones subsecuentes son rápidas (~30 seg - 2 min)
- Iconos actuales son placeholders - reemplazar antes de release

## 🎯 Estado

- ✅ Tauri CLI v1.6.3 instalado
- ✅ Configuración completa
- ✅ Listo para desarrollo y build
