# SearXNG Temas Personalizados

Esta carpeta contiene los temas para SearXNG. Está montada en el contenedor Docker en `/usr/local/searxng/searx/templates/`.

## 🎨 Temas disponibles

### **modern** ⭐ (NUEVO)
Tema completamente personalizado, moderno y atractivo con:
- ✨ Diseño limpio y minimalista
- 🌙 Modo claro/oscuro automático (toggle en navbar)
- 🎯 Bordes redondeados en todos los elementos
- 💫 Transiciones suaves y animaciones modernas
- 📱 Responsive design (funciona perfectamente en móvil)
- 🎨 Colores modernos y contraste perfecto
- ⚡ Shadow effects y efectos hover elegantes
- 🔧 CSS con variables CSS para fácil personalización

**Características principales:**
- **Light Mode**: Blanco limpio con azul como color principal
- **Dark Mode**: Negro profundo con azul más brillante
- **Toggle automático**: Detecta preferencia del sistema y permite cambiar manualmente
- **Botones redondeados**: 6px, 12px, 16px según tamaño
- **Espaciado consistente**: Sistema de spacing predefinido
- **Focus states**: Accesibilidad mejorada
- **Sombras dinámicas**: Diferentes niveles según importancia

### simple
Tema minimalista predeterminado de SearXNG

## 🚀 Cómo usar el tema modern

1. **Accede a SearXNG**: http://localhost:8080

2. **Cambia de tema**:
   - Ve a `Preferences` (engranaje en la esquina superior derecha)
   - En la sección "Theme", selecciona **"modern"**
   - Guarda los cambios

3. **Alterna entre modo claro/oscuro**:
   - Haz clic en el botón 🌙/☀️ en la barra superior
   - La preferencia se guarda automáticamente en el navegador

## 📝 Estructura del tema modern

```
modern/
├── base.html              # Plantilla base con CSS embebido y tema toggle
├── results.html           # Página de resultados de búsqueda
├── search.html            # Página de búsqueda principal
├── preferences.html       # Página de preferencias
├── 404.html              # Página de error 404
├── info.html             # Página de información
├── categories.html        # Selector de categorías
├── icons.html            # Sistema de iconos
├── macros.html           # Macros reutilizables
├── elements/             # Elementos reutilizables
│   ├── answers.html
│   ├── infobox.html
│   ├── suggestions.html
│   └── engines_msg.html
├── answer/               # Componentes de respuestas
├── filters/              # Plantillas de filtros
├── messages/             # Mensajes del sistema
├── preferences/          # Componentes de preferencias
├── result_templates/     # Plantillas de resultados específicas
└── README.md            # Documentación
```

## 🎨 Personalización del tema modern

Para modificar los colores, sombras o espaciado del tema modern, edita el archivo `base.html` en la sección `<style>`:

```css
:root {
  --radius-sm: 6px;           /* Cambiar bordes pequeños */
  --radius-md: 12px;          /* Cambiar bordes medianos */
  --radius-lg: 16px;          /* Cambiar bordes grandes */
  --accent-color: #2563eb;    /* Color principal (light mode) */
  --accent-hover: #1d4ed8;    /* Color hover (light mode) */
}

html[data-theme="light"] {
  --bg-primary: #ffffff;      /* Fondo principal light mode */
  --text-primary: #1a1a1a;    /* Texto principal light mode */
}

html[data-theme="dark"] {
  --bg-primary: #1a1a1a;      /* Fondo principal dark mode */
  --text-primary: #ffffff;    /* Texto principal dark mode */
}
```

## 🆕 Cómo agregar un nuevo tema personalizado

1. **Crea una carpeta para tu tema**:
   ```bash
   mkdir local-themes/mi-tema-bonito
   ```

2. **Copia la estructura del tema simple o modern como base**:
   ```bash
   cp -r local-themes/modern/* local-themes/mi-tema-bonito/
   ```

3. **Personaliza los archivos**:
   - `base.html` - Plantilla principal con estilos
   - `results.html` - Página de resultados
   - `search.html` - Página de búsqueda
   - `preferences.html` - Página de preferencias
   - Las carpetas `/elements`, `/answer`, `/filters`, etc.

4. **Los cambios se aplican automáticamente** sin reiniciar el contenedor

5. **Selecciona el tema en preferencias** de SearXNG

## 📐 Estructura general de temas

```
tu-tema/
├── base.html              # Plantilla base HTML
├── results.html           # Página de resultados de búsqueda
├── search.html            # Página de búsqueda principal
├── preferences.html       # Página de preferencias
├── 404.html              # Página de error 404
├── info.html             # Página de información
├── categories.html        # Selector de categorías
├── elements/             # Elementos reutilizables
├── answer/               # Componentes de respuestas
├── filters/              # Plantillas de filtros
├── messages/             # Mensajes del sistema
├── preferences/          # Componentes de preferencias
├── result_templates/     # Plantillas de resultados específicas
└── static/               # Assets estáticos (CSS, JS, imágenes)
```

## 🔧 Cómo funcionan los temas en SearXNG

- Los temas están en `/searx/templates/` dentro del contenedor
- SearXNG detecta automáticamente todas las carpetas como temas disponibles
- El nombre de la carpeta es el nombre del tema (ej: `modern`, `simple`)
- Los cambios se aplican sin reiniciar si modificas los archivos en el volumen montado
- La preferencia se guarda en las cookies/preferencias de SearXNG

## 📚 Documentación

Para más información sobre cómo crear temas para SearXNG:
- **Repositorio oficial**: https://github.com/searxng/searxng
- **Documentación de temas**: https://docs.searxng.org/
- **Documentación de templates**: https://docs.searxng.org/admin/settings/index.html

## 💡 Notas importantes

- El volumen está configurado en `docker-compose.yml` apuntando a `./local-themes`
- Los cambios se aplican inmediatamente sin necesidad de reiniciar el contenedor
- Los permisos del volumen son `rw` (lectura y escritura)
- La preferencia de tema (light/dark) en el tema **modern** se guarda en `localStorage`
- Los temas heredan los estilos base de SearXNG (`sxng-ltr.min.css` o `sxng-rtl.min.css`)

## 🎯 Tips para crear temas atractivos

1. **Usa CSS Variables** para fácil personalización
2. **Implementa dark mode** con `prefers-color-scheme` media query
3. **Agrega transiciones suaves** para mejor UX
4. **Usa bordes redondeados** para aspecto moderno
5. **Sombras sutiles** para profundidad visual
6. **Responsive design** desde móvil hasta desktop
7. **Accesibilidad**: Focus states, contraste de colores, etc.
8. **Animaciones** para micro-interacciones (hover, click)
