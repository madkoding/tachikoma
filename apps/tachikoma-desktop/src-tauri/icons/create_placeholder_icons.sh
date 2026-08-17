#!/bin/bash
# Crear iconos placeholder para Tauri
# Estos son temporales - reemplazar con iconos reales

# Verificar que imagemagick esté instalado
if ! command -v convert &> /dev/null; then
    echo "❌ ImageMagick no está instalado. Instalando..."
    sudo apt-get update && sudo apt-get install -y imagemagick
fi

# Crear un icono base simple (un círculo azul con "N" en el centro)
convert -size 1024x1024 xc:none \
    -fill "#6366f1" -draw "circle 512,512 512,100" \
    -fill white -font DejaVu-Sans-Bold -pointsize 600 \
    -gravity center -annotate +0+0 "N" \
    base-1024.png

# Generar los diferentes tamaños
convert base-1024.png -resize 32x32 32x32.png
convert base-1024.png -resize 128x128 128x128.png
convert base-1024.png -resize 256x256 128x128@2x.png

# Generar .ico para Windows (múltiples tamaños en un archivo)
convert base-1024.png -define icon:auto-resize=256,128,64,48,32,16 icon.ico

# Para .icns necesitamos estar en macOS, por ahora creamos un placeholder
cp base-1024.png icon.icns

echo "✅ Iconos placeholder creados"
ls -lh *.png *.ico *.icns
