# Voice Service (Rust) - NEURO-Voice

Servicio de síntesis de voz ultra-rápido usando Piper TTS con efectos de audio estilo **Tachikoma** (Ghost in the Shell).

## Perfil de Voz Tachikoma

La voz está configurada para sonar como un Tachikoma de Ghost in the Shell: Stand Alone Complex:

- **Voz aguda/infantil** - Pitch elevado (+6 semitonos)
- **Velocidad entusiasta** - 1.05x más rápido
- **Tono metálico sutil** - Ring modulation ligero a 80Hz
- **Claridad** - High-pass a 800Hz elimina graves
- **Procesamiento IA** - Chorus suave de 2 voces
- **Cuerpo robótico** - Reverb pequeño como si hablara desde dentro de su carcasa

## Características

- **Motor TTS**: Piper TTS (ONNX) - síntesis local de alta velocidad
- **Efectos de Audio**:
  - Cambio de tono (pitch shift)
  - Filtro high-pass
  - Chorus (3 voces)
  - Flanger
  - Ring modulation
  - Reverb
- **Limpieza de Texto**: Elimina emojis, código, markdown, URLs
- **Streaming**: Síntesis por oraciones vía SSE
- **API REST**: Compatible con la API de Python original

## API Endpoints

| Método | Ruta | Descripción |
|--------|------|-------------|
| GET | `/` | Información del servicio |
| GET | `/health` | Estado de salud |
| GET | `/status` | Estado detallado |
| GET | `/voices` | Lista de voces disponibles |
| POST | `/synthesize` | Sintetizar texto a audio WAV |
| POST | `/synthesize/stream` | Síntesis con streaming SSE |

## Uso

### Síntesis básica

```bash
curl -X POST http://localhost:8100/synthesize \
  -H "Content-Type: application/json" \
  -d '{"text": "Hola mundo", "voice": "es_MX-claude-high"}' \
  --output speech.wav
```

### Síntesis con efectos

```bash
curl -X POST http://localhost:8100/synthesize \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Hola, soy un robot",
    "voice": "es_MX-claude-high",
    "speed": 1.0,
    "pitch_shift": 2.0,
    "robot_effect": true
  }' \
  --output robot_speech.wav
```

### Síntesis sin efectos

```bash
curl -X POST http://localhost:8100/synthesize \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Voz natural sin efectos",
    "robot_effect": false,
    "pitch_shift": 0.0
  }' \
  --output natural_speech.wav
```

## Configuración

Variables de entorno:

| Variable | Descripción | Default |
|----------|-------------|---------|
| `HOST` | Dirección de escucha | `0.0.0.0` |
| `PORT` | Puerto del servidor | `8100` |
| `PIPER_BIN` | Ruta al binario de Piper | `/app/piper/piper` |
| `MODELS_DIR` | Directorio de modelos | `/app/models` |
| `DEFAULT_VOICE` | Voz por defecto | `es_MX-claude-high` |
| `RUST_LOG` | Nivel de logging | `info` |

## Desarrollo

### Requisitos

- Rust 1.70+
- Piper TTS binario
- Modelos de voz Piper (.onnx + .onnx.json)

### Compilar

```bash
cargo build --release
```

### Ejecutar

```bash
# Configurar variables de entorno
export PIPER_BIN=/path/to/piper
export MODELS_DIR=/path/to/models

# Ejecutar
cargo run --release
```

### Tests

```bash
cargo test
```

## Docker

### Construir imagen

```bash
docker build -t voice-service-rust .
```

### Ejecutar contenedor

```bash
docker run -p 8100:8100 voice-service-rust
```

## Voces Disponibles

El servicio detecta automáticamente las voces en el directorio de modelos. Cada voz requiere:

- `{voice_name}.onnx` - Modelo ONNX
- `{voice_name}.onnx.json` - Configuración

Descargar voces de: https://huggingface.co/rhasspy/piper-voices

## Parámetros de Efectos (Perfil Tachikoma)

### Pitch Shift
- `pitch_shift`: Semitonos a subir/bajar el tono
  - `0.0` = Sin cambio
  - `6.0` = 6 semitonos arriba (default Tachikoma - voz infantil)
  - `12.0` = Una octava arriba

### Robot Effect
- `robot_effect`: `true` | `false`
  - Aplica cadena de efectos optimizada para voz Tachikoma:
    - High-pass (800Hz) → Chorus (2 voces) → Flanger sutil → Ring mod (80Hz) → Reverb pequeño

### Speed
- `speed`: Multiplicador de velocidad
  - `1.0` = Normal
  - `1.05` = Default Tachikoma (ligeramente entusiasta)
  - `1.2` = 20% más rápido

### Parámetros Avanzados (en código)

| Parámetro | Valor Tachikoma | Descripción |
|-----------|-----------------|-------------|
| `highpass_cutoff` | 800 Hz | Elimina graves para voz "pequeña" |
| `chorus_wet` | 0.25 | Mezcla sutil de chorus |
| `chorus_voices` | 2 | Voces de chorus |
| `flanger_rate` | 0.3 Hz | Velocidad del LFO |
| `flanger_depth` | 0.08 | Profundidad del efecto |
| `flanger_wet` | 0.08 | Mezcla muy sutil |
| `ring_freq` | 80 Hz | Frecuencia para brillo metálico |
| `ring_wet` | 0.02 | Apenas perceptible |
| `reverb_wet` | 0.12 | Reverb de cuerpo robótico |
| `reverb_decay` | 0.3s | Decay corto |
| `reverb_room_size` | 0.08 | Espacio pequeño |

## Licencia

MIT
