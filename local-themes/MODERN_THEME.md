# 🎨 Modern SearXNG Theme

Un tema moderno, minimalista y atractivo para SearXNG con soporte completo para modo claro/oscuro.

## ✨ Características

### Diseño Moderno
- **Bordes redondeados** en todos los elementos (6px, 12px, 16px)
- **Sombras elegantes** con diferentes niveles de profundidad
- **Espaciado consistente** usando variables CSS
- **Transiciones suaves** para todas las interacciones
- **Animaciones fluidas** en hover y focus states

### Tema Claro/Oscuro
- 🌙 **Toggle automático** en la barra superior
- ☀️ **Detecta preferencia del sistema** (prefers-color-scheme)
- 💾 **Guarda preferencia** en localStorage
- 🎨 **Paleta de colores completa** para ambos modos
  - **Light**: Blanco, grises claros, azul vibrante
  - **Dark**: Negro profundo, grises oscuros, azul brillante

### Responsive Design
- 📱 **Mobile-first** approach
- 🖥️ **Desktop optimizado** para máxima legibilidad
- 📐 **Breakpoints** en 768px y 480px
- 📊 **Grid layout** que se adapta automáticamente

### Accesibilidad
- ♿ **Focus states** mejorados
- 🎯 **Contraste óptimo** en ambos modos
- ⌨️ **Navegación por teclado** completamente funcional
- 🎬 **Respeta prefers-reduced-motion**

## 🚀 Cómo usar

### Activar el tema
1. Ve a http://localhost:8080
2. Abre **Preferences** (⚙️ en la esquina superior derecha)
3. En la sección "Theme", selecciona **"modern"**
4. Guarda los cambios

### Cambiar entre claro y oscuro
1. Haz clic en el botón 🌙 o ☀️ en la barra superior
2. El modo se cambia automáticamente
3. La preferencia se guarda en tu navegador

## 🎯 Elementos estilizados

### Barra de navegación
- Fondo degradado sutil
- Links con hover effect
- Toggle de tema integrado
- Responsive con collapse de texto en móvil

### Búsqueda
- Input con borde redondeado
- Focus state con sombra azul
- Botones clear y search con hover interactivos
- Autocomplete elegante

### Categorías
- Botones con borde redondeado
- Active state con fondo azul
- Hover effects suaves
- Responsive en móvil

### Resultados de búsqueda
- Tarjetas con hover lift (transform translateY)
- Bordes redondeados y sombras sutiles
- Título en azul clickeable
- URL en gris oscuro
- Descripción clara y legible

### Sidebar
- Infobox con border-left azul
- Suggestions con hover effect
- Collapsible details elegante
- Responsive stacking en móvil

### Footer
- Texto centrado y discreto
- Links con color tema
- Borde superior sutil

## 🎨 Paleta de colores

### Modo Claro (Light)
```
Background Primary:    #ffffff (Blanco puro)
Background Secondary:  #f8f9fa (Gris muy claro)
Background Tertiary:   #f0f2f5 (Gris claro)
Text Primary:          #1a1a1a (Negro casi puro)
Text Secondary:        #4a4a4a (Gris oscuro)
Text Tertiary:         #7a7a7a (Gris medio)
Accent Color:          #2563eb (Azul vibrante)
Accent Hover:          #1d4ed8 (Azul más oscuro)
Border Color:          #e5e7eb (Gris muy claro)
```

### Modo Oscuro (Dark)
```
Background Primary:    #1a1a1a (Negro profundo)
Background Secondary:  #2d2d2d (Gris muy oscuro)
Background Tertiary:   #3d3d3d (Gris oscuro)
Text Primary:          #ffffff (Blanco puro)
Text Secondary:        #e0e0e0 (Gris muy claro)
Text Tertiary:         #b0b0b0 (Gris claro)
Accent Color:          #3b82f6 (Azul brillante)
Accent Hover:          #60a5fa (Azul más claro)
Border Color:          #404040 (Gris oscuro)
```

## 🔧 Personalización

### Cambiar colores de acento
Edita `base.html` y busca la sección `<style>`:

```css
/* Light mode */
html[data-theme="light"] {
  --accent-color: #2563eb;    /* Cambia este color */
  --accent-hover: #1d4ed8;    /* Y este */
}

/* Dark mode */
html[data-theme="dark"] {
  --accent-color: #3b82f6;    /* Cambia este color */
  --accent-hover: #60a5fa;    /* Y este */
}
```

### Cambiar bordes redondeados
```css
:root {
  --radius-sm: 6px;   /* Bordes pequeños */
  --radius-md: 12px;  /* Bordes medianos */
  --radius-lg: 16px;  /* Bordes grandes */
}
```

### Cambiar espaciado
```css
:root {
  --spacing-xs: 4px;    /* Extra pequeño */
  --spacing-sm: 8px;    /* Pequeño */
  --spacing-md: 16px;   /* Mediano */
  --spacing-lg: 24px;   /* Grande */
  --spacing-xl: 32px;   /* Extra grande */
}
```

### Cambiar velocidad de transiciones
```css
:root {
  --transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1); /* Cambia 0.3s */
}
```

## 📱 Responsive Breakpoints

### Desktop (> 768px)
- Grid de resultados: 1fr 300px
- Sidebar visible al lado
- Navegación completa con textos

### Tablet (768px - 481px)
- Grid de resultados: 1fr (stack vertical)
- Sidebar debajo del contenido
- Navegación con textos colapsados

### Mobile (< 480px)
- Font size reducido a 14px
- Barra de navegación vertical
- Toggle de tema a ancho completo
- Resultados simplificados

## 🎬 Animaciones y Transiciones

### Hover Effects
- Links: Color change suave
- Botones: Background color + shadow lift
- Tarjetas (results): Border color + shadow + translateY(-2px)
- Icons: Rotate smooth en hover

### Focus States
- Outline de 2px en color acento
- Offset de 2px
- Respeta prefers-reduced-motion

### Loading/Transitions
- Suave cambio entre light/dark (color transition)
- Animaciones de slide-in para mensajes de error
- Cubic bezier easing: `cubic-bezier(0.4, 0, 0.2, 1)`

## ⌨️ Accesibilidad

### Keyboard Navigation
- Tab through all interactive elements
- Enter/Space para activar botones
- Escape para cerrar autocomplete
- Focus visible en todos los elementos

### Color Contrast
- Light mode: Ratio mínimo 4.5:1
- Dark mode: Ratio mínimo 4.5:1
- Probado con WCAG AA

### Reducción de Movimiento
- Media query `prefers-reduced-motion: reduce`
- Desactiva animaciones si el usuario lo requiere

## 📊 Estadísticas del Tema

- **Archivos HTML**: 30+
- **Líneas de CSS**: ~600 (embebido en base.html)
- **JavaScript**: ~50 líneas (toggle de tema)
- **Tamaño total**: ~150KB (sin comprimir)
- **Compatibilidad**: Todos los navegadores modernos

## 🌐 Compatibilidad

- ✅ Chrome 88+
- ✅ Firefox 85+
- ✅ Safari 14+
- ✅ Edge 88+
- ✅ Opera 74+
- ✅ Mobile browsers (iOS Safari, Chrome Android)

## 🐛 Debugging

Si el tema no carga correctamente:

1. **Verifica que el tema esté en el container**:
   ```bash
   docker exec neuro-searxng ls -la /usr/local/searxng/searx/templates/modern/
   ```

2. **Revisa los logs del container**:
   ```bash
   docker logs neuro-searxng | tail -50
   ```

3. **Limpia la caché del navegador**: Ctrl+Shift+Del

4. **Reinicia el contenedor**:
   ```bash
   docker-compose restart searxng
   ```

## 📝 Notas técnicas

- **CSS**: Embebido en `base.html` para facilitar la distribución
- **JavaScript**: Mínimo necesario para el toggle de tema
- **localStorage**: Usado para guardar preferencia de tema
- **CSS Variables**: Completa portabilidad entre modos
- **Herencia**: Extiende los estilos base de SearXNG

## 🤝 Contribuciones

Para mejorar el tema:
1. Haz una copia: `cp -r modern mi-tema-mejorado`
2. Personaliza los estilos en `base.html`
3. Prueba cambios en vivo
4. Comparte tus mejoras

## 📖 Recursos

- [SearXNG GitHub](https://github.com/searxng/searxng)
- [SearXNG Docs](https://docs.searxng.org/)
- [CSS Variables MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/--*)
- [prefers-color-scheme MDN](https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-color-scheme)

---

Disfrutá del tema moderno! 🎨✨
