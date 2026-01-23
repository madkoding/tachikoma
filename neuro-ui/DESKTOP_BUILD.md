# NEURO-OS Desktop - Build Guide

Esta guía explica cómo compilar NEURO-OS UI como aplicación de escritorio para Windows, Linux y macOS usando Tauri.

## 📋 Prerrequisitos

### Todos los sistemas operativos

1. **Node.js** (v18 o superior)
2. **Rust** (versión estable más reciente)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

### Linux (Ubuntu/Debian)

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

### macOS

```bash
xcode-select --install
```

### Windows

1. **Microsoft Visual Studio C++ Build Tools**
   - Descargar de: https://visualstudio.microsoft.com/visual-cpp-build-tools/
   - Instalar "Desktop development with C++"

2. **WebView2** (usualmente ya está instalado en Windows 10/11)
   - Si no: https://developer.microsoft.com/en-us/microsoft-edge/webview2/

## 🚀 Instalación

```bash
cd neuro-ui

# Instalar dependencias de npm
npm install

# Instalar CLI de Tauri (si no está instalado globalmente)
npm install -g @tauri-apps/cli
```

## 🛠️ Desarrollo

Modo desarrollo con hot-reload:

```bash
npm run tauri:dev
```

Esto:
1. Inicia el servidor de desarrollo Vite
2. Lanza la aplicación Tauri
3. Recarga automáticamente al cambiar código

## 📦 Compilación para Producción

### Linux (actual sistema)

```bash
# Build para Linux x86_64
npm run tauri:build:linux

# O simplemente:
npm run tauri:build
```

Salida: `src-tauri/target/release/bundle/`
- `.deb` - Paquete Debian/Ubuntu
- `.AppImage` - Portable para cualquier distro Linux

### Windows (cross-compilation desde Linux)

```bash
# Instalar target de Windows
rustup target add x86_64-pc-windows-msvc

# Build
npm run tauri:build:windows
```

**Nota**: Cross-compilation de Linux a Windows puede requerir dependencias adicionales. Es más confiable compilar en Windows nativo.

### macOS (cross-compilation desde Linux)

```bash
# Instalar target de macOS
rustup target add aarch64-apple-darwin  # Mac M1/M2
rustup target add x86_64-apple-darwin   # Mac Intel

# Build
npm run tauri:build:mac
```

**Nota**: Cross-compilation de Linux a macOS requiere herramientas especiales (osxcross) y es complejo. Se recomienda compilar en macOS nativo.

## 🏗️ Compilación Nativa por Plataforma

### En Linux

```bash
npm run tauri:build
```

Genera:
- `.deb` para Debian/Ubuntu
- `.AppImage` portable
- `.tar.gz` con binario

### En Windows

```bash
npm run tauri:build
```

Genera:
- `.msi` instalador Windows
- `.exe` portable (en `/bundle/nsis/`)

### En macOS

```bash
npm run tauri:build
```

Genera:
- `.dmg` instalador macOS
- `.app` bundle

## 📁 Ubicación de Builds

Todos los builds se generan en:
```
neuro-ui/src-tauri/target/release/bundle/
├── deb/          # Linux .deb
├── appimage/     # Linux .AppImage
├── msi/          # Windows .msi
├── nsis/         # Windows .exe portable
└── dmg/          # macOS .dmg
```

## 🎨 Iconos

Los iconos de la aplicación están en `src-tauri/icons/`. Ver [src-tauri/icons/README.md](src-tauri/icons/README.md) para instrucciones sobre cómo generarlos.

## ⚙️ Configuración

La configuración de Tauri está en [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json):

- **Tamaño de ventana**: 1200x800 (mínimo 800x600)
- **Identificador**: `com.neuroos.ui`
- **Nombre**: NEURO-OS

## 🔧 Solución de Problemas

### Error: "Failed to bundle project"

Verificar que todas las dependencias del sistema estén instaladas.

### Error de WebView2 en Windows

Instalar WebView2 Runtime: https://go.microsoft.com/fwlink/p/?LinkId=2124703

### Error de linkado en Linux

```bash
sudo apt install libwebkit2gtk-4.0-dev
```

### Build muy lento

Los primeros builds son lentos porque compilan todas las dependencias de Rust. Builds subsecuentes son mucho más rápidos.

## 📚 Recursos

- [Tauri Documentation](https://tauri.app/)
- [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)
- [Tauri Building Guide](https://tauri.app/v1/guides/building/)

## 🚢 Distribución

### Linux

Distribuir `.AppImage` (portable) o `.deb` (para Debian/Ubuntu).

### Windows

Distribuir `.msi` (instalador) o `.exe` desde `nsis/` (portable).

### macOS

Distribuir `.dmg`. Para distribución en Mac App Store, requiere firma con certificado de desarrollador Apple.

## 🔐 Firma de Código

Para distribución pública, se recomienda firmar los ejecutables:

- **Windows**: Certificado de Authenticode
- **macOS**: Apple Developer ID
- **Linux**: Opcional, pero recomendado para repos oficiales

Ver [Tauri Code Signing](https://tauri.app/v1/guides/distribution/sign-macos) para más detalles.
