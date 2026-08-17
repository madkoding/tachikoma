# Iconos de NEURO-OS

Esta carpeta debe contener los iconos de la aplicación en diferentes formatos:

- `32x32.png` - Icono para Windows (32x32px)
- `128x128.png` - Icono para Linux (128x128px)
- `128x128@2x.png` - Icono para macOS Retina (256x256px)
- `icon.icns` - Icono para macOS (generado desde PNG)
- `icon.ico` - Icono para Windows (generado desde PNG)

## Generar iconos

Puedes usar estas herramientas:

```bash
# Instalar imagemagick para convertir imágenes
sudo apt install imagemagick  # Linux
brew install imagemagick      # macOS

# Desde un PNG de 1024x1024px:
convert icon-1024.png -resize 32x32 32x32.png
convert icon-1024.png -resize 128x128 128x128.png
convert icon-1024.png -resize 256x256 128x128@2x.png

# Para .ico (Windows):
convert icon-1024.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico

# Para .icns (macOS) - necesitas iconutil en macOS:
# Crear icon.iconset/ con los tamaños requeridos y ejecutar:
# iconutil -c icns icon.iconset
```

O usar servicios online como:
- https://www.icoconverter.com/
- https://cloudconvert.com/
