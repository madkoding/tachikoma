"""
Voice Service - Piper TTS API
==============================
FastAPI service for text-to-speech synthesis using Piper TTS.
Ultra-fast local TTS with ONNX runtime.
"""

import io
import json
import logging
import re
import subprocess
import wave
from contextlib import asynccontextmanager
from pathlib import Path

import numpy as np
import soundfile as sf
from fastapi import FastAPI, HTTPException, Response
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import StreamingResponse
from pydantic import BaseModel

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Piper executable and model paths
PIPER_DIR = Path("/app/piper")
PIPER_BIN = PIPER_DIR / "piper"
MODELS_DIR = Path("/app/models")

# Available voices (will be populated on startup)
AVAILABLE_VOICES: dict = {}

# Default voice
DEFAULT_VOICE = "es_MX-claude-high"

# Emoji pattern (covers most common emoji ranges)
EMOJI_PATTERN = re.compile(
    "["
    "\U0001F600-\U0001F64F"  # emoticons
    "\U0001F300-\U0001F5FF"  # symbols & pictographs
    "\U0001F680-\U0001F6FF"  # transport & map symbols
    "\U0001F1E0-\U0001F1FF"  # flags
    "\U00002702-\U000027B0"  # dingbats
    "\U000024C2-\U0001F251"  # enclosed characters
    "\U0001F900-\U0001F9FF"  # supplemental symbols
    "\U0001FA00-\U0001FA6F"  # chess symbols
    "\U0001FA70-\U0001FAFF"  # symbols extended
    "\U00002600-\U000026FF"  # misc symbols
    "\U00002300-\U000023FF"  # misc technical
    "]+",
    flags=re.UNICODE
)


def clean_text_for_speech(text: str) -> str:
    """
    Clean text for speech synthesis by removing:
    - Emojis
    - Code blocks (```...```)
    - Inline code (`...`)
    - Markdown formatting
    - URLs
    - Extra whitespace
    """
    # Remove code blocks (```language\n...\n```)
    text = re.sub(r'```[\s\S]*?```', ' código omitido ', text)
    
    # Remove inline code (`...`)
    text = re.sub(r'`[^`]+`', '', text)
    
    # Remove emojis
    text = EMOJI_PATTERN.sub('', text)
    
    # Remove URLs
    text = re.sub(r'https?://\S+', '', text)
    
    # Remove markdown bold/italic
    text = re.sub(r'\*\*([^*]+)\*\*', r'\1', text)  # **bold**
    text = re.sub(r'\*([^*]+)\*', r'\1', text)      # *italic*
    text = re.sub(r'__([^_]+)__', r'\1', text)      # __bold__
    text = re.sub(r'_([^_]+)_', r'\1', text)        # _italic_
    
    # Remove markdown headers
    text = re.sub(r'^#{1,6}\s+', '', text, flags=re.MULTILINE)
    
    # Remove markdown list markers
    text = re.sub(r'^\s*[-*+]\s+', '', text, flags=re.MULTILINE)
    text = re.sub(r'^\s*\d+\.\s+', '', text, flags=re.MULTILINE)
    
    # Remove multiple spaces and newlines
    text = re.sub(r'\s+', ' ', text)
    
    # Trim
    text = text.strip()
    
    return text


class SynthesizeRequest(BaseModel):
    """Request model for text-to-speech synthesis."""
    text: str
    voice: str = DEFAULT_VOICE
    speed: float = 1.0  # Speech rate multiplier
    pitch_shift: float = 3.0  # Pitch shift in semitones (default +3)
    robot_effect: bool = True  # Add robotic effect chain


class StatusResponse(BaseModel):
    """Status response model."""
    enabled: bool
    engine: str
    sample_rate: int
    available_voices: list[str]
    default_voice: str


def discover_voices():
    """Discover available Piper voice models."""
    global AVAILABLE_VOICES
    AVAILABLE_VOICES = {}
    
    if not MODELS_DIR.exists():
        logger.warning(f"Models directory not found: {MODELS_DIR}")
        return
    
    # Look for .onnx files
    for onnx_file in MODELS_DIR.glob("*.onnx"):
        voice_name = onnx_file.stem
        json_file = onnx_file.with_suffix(".onnx.json")
        
        if json_file.exists():
            AVAILABLE_VOICES[voice_name] = {
                "model": str(onnx_file),
                "config": str(json_file),
            }
            logger.info(f"✅ Found voice: {voice_name}")
    
    logger.info(f"📢 Total voices available: {len(AVAILABLE_VOICES)}")


def check_piper_installed() -> bool:
    """Check if Piper is installed and working."""
    if not PIPER_BIN.exists():
        logger.error(f"Piper binary not found at {PIPER_BIN}")
        return False
    
    try:
        subprocess.run(
            [str(PIPER_BIN), "--help"],
            capture_output=True,
            timeout=5
        )
        return True
    except Exception as e:
        logger.error(f"Piper check failed: {e}")
        return False


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Initialize on startup."""
    logger.info("🎙️ Voice Service (Piper TTS) starting...")
    
    # Check Piper installation
    if check_piper_installed():
        logger.info("✅ Piper TTS binary found")
    else:
        logger.warning("⚠️ Piper TTS not found - synthesis will fail")
    
    # Discover available voices
    discover_voices()
    
    if AVAILABLE_VOICES:
        logger.info(f"✅ Voice Service ready with {len(AVAILABLE_VOICES)} voices!")
    else:
        logger.warning("⚠️ No voice models found - please download models")
    
    yield
    logger.info("👋 Voice Service shutting down...")


app = FastAPI(
    title="Voice Service",
    description="Piper TTS API for NEURO-OS - Ultra-fast local TTS",
    version="3.0.0",
    lifespan=lifespan,
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/")
async def root():
    """Root endpoint."""
    return {
        "service": "voice",
        "status": "running",
        "engine": "piper-tts",
        "voices": len(AVAILABLE_VOICES)
    }


@app.get("/health")
async def health():
    """Health check endpoint."""
    piper_ok = check_piper_installed()
    has_voices = len(AVAILABLE_VOICES) > 0
    return {
        "status": "healthy" if piper_ok and has_voices else "degraded",
        "model_loaded": has_voices,
        "piper_installed": piper_ok,
        "voices_count": len(AVAILABLE_VOICES)
    }


@app.get("/status", response_model=StatusResponse)
async def get_status():
    """Get voice service status."""
    return StatusResponse(
        enabled=len(AVAILABLE_VOICES) > 0,
        engine="piper-tts",
        sample_rate=22050,
        available_voices=list(AVAILABLE_VOICES.keys()),
        default_voice=DEFAULT_VOICE,
    )


@app.get("/voices")
async def list_voices():
    """List available voices."""
    return {
        "voices": list(AVAILABLE_VOICES.keys()),
        "default": DEFAULT_VOICE
    }


@app.post("/synthesize")
async def synthesize(request: SynthesizeRequest):
    """
    Synthesize text to speech using Piper TTS.
    Returns WAV audio.
    """
    text = request.text.strip()
    
    if not text:
        raise HTTPException(status_code=400, detail="Text cannot be empty")
    
    # Clean text for speech (remove emojis, code blocks, markdown, etc.)
    text = clean_text_for_speech(text)
    
    if not text:
        raise HTTPException(status_code=400, detail="Text is empty after cleaning")
    
    if len(text) > 5000:
        text = text[:5000]
        logger.warning("Text truncated to 5000 characters")
    
    # Select voice
    voice = request.voice if request.voice in AVAILABLE_VOICES else DEFAULT_VOICE
    
    if voice not in AVAILABLE_VOICES:
        # If no voices available, return error
        if not AVAILABLE_VOICES:
            raise HTTPException(
                status_code=503,
                detail="No voice models available. Please download Piper models."
            )
        # Use first available voice
        voice = list(AVAILABLE_VOICES.keys())[0]
    
    voice_config = AVAILABLE_VOICES[voice]
    
    logger.info(f"🎤 Synthesizing {len(text)} chars with voice '{voice}'")
    
    try:
        # Call Piper via subprocess
        # Piper reads from stdin and outputs raw audio to stdout
        process = subprocess.Popen(
            [
                str(PIPER_BIN),
                "--model", voice_config["model"],
                "--config", voice_config["config"],
                "--output-raw",
                "--length-scale", str(1.0 / request.speed),  # Inverse for speed
            ],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
        
        # Send text and get audio
        audio_raw, stderr = process.communicate(input=text.encode("utf-8"), timeout=30)
        
        if process.returncode != 0:
            logger.error(f"Piper error: {stderr.decode()}")
            raise HTTPException(status_code=500, detail="Synthesis failed")
        
        # Convert raw audio to WAV
        # Piper outputs 16-bit signed PCM at 22050 Hz mono
        sample_rate = 22050
        
        # Convert raw bytes to numpy array
        audio_array = np.frombuffer(audio_raw, dtype=np.int16).astype(np.float32) / 32768.0
        
        # Apply pitch shift using fast resampling method
        if request.pitch_shift != 0:
            logger.info(f"🎵 Applying pitch shift: {request.pitch_shift} semitones")
            # Fast pitch shift via resampling
            # Shift factor: 2^(semitones/12)
            shift_factor = 2 ** (request.pitch_shift / 12)
            # Resample to change pitch (also changes duration)
            new_length = int(len(audio_array) / shift_factor)
            indices = np.linspace(0, len(audio_array) - 1, new_length)
            audio_array = np.interp(indices, np.arange(len(audio_array)), audio_array)
        
        # Apply robot effect chain
        if request.robot_effect:
            logger.info("🤖 Applying robot effect chain")
            t = np.arange(len(audio_array)) / sample_rate
            
            # 1. High-Pass Filter (500Hz, 12dB/oct approximation with 2-pass)
            cutoff_freq = 500
            rc = 1.0 / (2 * np.pi * cutoff_freq)
            dt = 1.0 / sample_rate
            alpha = rc / (rc + dt)
            
            # First pass
            filtered = np.zeros_like(audio_array)
            filtered[0] = audio_array[0]
            for i in range(1, len(audio_array)):
                filtered[i] = alpha * (filtered[i-1] + audio_array[i] - audio_array[i-1])
            # Second pass for steeper slope
            audio_hp = np.zeros_like(filtered)
            audio_hp[0] = filtered[0]
            for i in range(1, len(filtered)):
                audio_hp[i] = alpha * (audio_hp[i-1] + filtered[i] - filtered[i-1])
            
            audio_array = audio_hp
            
            # 2. Chorus (3 voces, 8ms delay, 25% depth, 1.5Hz LFO, 40% wet)
            chorus_wet = 0.4
            chorus_voices = []
            for voice_idx in range(3):
                lfo_rate = 1.5 + voice_idx * 0.3  # Slightly different rates
                delay_ms = 8 + voice_idx * 2  # 8ms, 10ms, 12ms
                depth = 0.25
                
                max_delay_samples = int(sample_rate * delay_ms / 1000)
                lfo = (1 + np.sin(2 * np.pi * lfo_rate * t + voice_idx * np.pi * 2 / 3)) / 2
                delay_samples = (lfo * max_delay_samples * depth).astype(int) + max_delay_samples // 2
                
                voice_out = np.zeros_like(audio_array)
                for i in range(len(audio_array)):
                    delay = delay_samples[i]
                    if i >= delay:
                        voice_out[i] = audio_array[i - delay]
                chorus_voices.append(voice_out)
            
            chorus_mix = sum(chorus_voices) / 3
            audio_array = audio_array * (1 - chorus_wet) + chorus_mix * chorus_wet
            
            # 3. Flanger (0.5Hz rate, 15% depth, 20% feedback, 15% wet)
            flanger_rate = 0.5
            flanger_depth = 0.15
            flanger_feedback = 0.2
            flanger_wet = 0.15
            max_flanger_delay = int(sample_rate * 0.007)  # 7ms max
            
            lfo = (1 + np.sin(2 * np.pi * flanger_rate * t)) / 2
            delay_samples = (lfo * max_flanger_delay * flanger_depth).astype(int)
            
            flanged = np.zeros_like(audio_array)
            feedback_buffer = np.zeros_like(audio_array)
            for i in range(len(audio_array)):
                delay = max(1, delay_samples[i])
                if i >= delay:
                    flanged[i] = audio_array[i - delay] + feedback_buffer[i - delay] * flanger_feedback
                    feedback_buffer[i] = flanged[i]
            
            audio_array = audio_array * (1 - flanger_wet) + flanged * flanger_wet
            
            # 4. Ring Modulation (50Hz carrier, sinusoidal, 3% wet)
            ring_freq = 50
            ring_wet = 0.03
            ring_mod = np.sin(2 * np.pi * ring_freq * t)
            ring_signal = audio_array * ring_mod
            audio_array = audio_array * (1 - ring_wet) + ring_signal * ring_wet
            
            # 5. Small Room Reverb (10% size, 0.5s decay, 10% wet)
            reverb_wet = 0.10
            decay_time = 0.5
            room_size = 0.10
            
            # Simple comb filter reverb approximation
            delay_times_ms = [23, 29, 37, 43]  # Prime numbers for less metallic sound
            reverb_out = np.zeros_like(audio_array)
            
            for delay_ms in delay_times_ms:
                delay_samples_rev = int(sample_rate * delay_ms * room_size / 1000)
                if delay_samples_rev < 1:
                    delay_samples_rev = 1
                decay = np.exp(-3 * delay_ms * room_size / 1000 / decay_time)
                
                delayed = np.zeros_like(audio_array)
                for i in range(delay_samples_rev, len(audio_array)):
                    delayed[i] = audio_array[i - delay_samples_rev] * decay
                reverb_out += delayed
            
            reverb_out /= len(delay_times_ms)
            audio_array = audio_array * (1 - reverb_wet) + reverb_out * reverb_wet
            
            # Normalize to prevent clipping
            max_val = np.max(np.abs(audio_array))
            if max_val > 0.95:
                audio_array = audio_array * 0.95 / max_val
        
        # Write to WAV buffer
        buffer = io.BytesIO()
        sf.write(buffer, audio_array, sample_rate, format='WAV', subtype='PCM_16')
        buffer.seek(0)
        wav_bytes = buffer.read()
        
        logger.info(f"✅ Generated {len(wav_bytes)} bytes of audio")
        
        return Response(
            content=wav_bytes,
            media_type="audio/wav",
            headers={
                "Content-Disposition": "inline; filename=speech.wav"
            }
        )
        
    except subprocess.TimeoutExpired:
        logger.error("Piper synthesis timed out")
        raise HTTPException(status_code=504, detail="Synthesis timed out")
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Synthesis error: {e}")
        raise HTTPException(status_code=500, detail=str(e))


def split_into_sentences(text: str) -> list[str]:
    """Split text into sentences for streaming synthesis."""
    # Split on sentence-ending punctuation
    sentences = re.split(r'(?<=[.!?])\s+', text)
    # Filter empty and merge very short sentences
    result = []
    for s in sentences:
        s = s.strip()
        if not s:
            continue
        # If sentence is very short, merge with previous
        if result and len(s) < 20 and len(result[-1]) < 100:
            result[-1] = result[-1] + ' ' + s
        else:
            result.append(s)
    return result if result else [text]


@app.post("/synthesize/stream")
async def synthesize_stream(request: SynthesizeRequest):
    """
    Synthesize text to speech with streaming response.
    Sends audio sentence by sentence as base64-encoded WAV chunks via SSE.
    """
    text = clean_text_for_speech(request.text.strip())
    
    if not text:
        raise HTTPException(status_code=400, detail="Text cannot be empty")
    
    voice = request.voice if request.voice in AVAILABLE_VOICES else DEFAULT_VOICE
    
    if voice not in AVAILABLE_VOICES:
        if not AVAILABLE_VOICES:
            raise HTTPException(status_code=503, detail="No voice models available")
        voice = list(AVAILABLE_VOICES.keys())[0]
    
    voice_config = AVAILABLE_VOICES[voice]
    sentences = split_into_sentences(text)
    
    logger.info(f"🎤 Streaming {len(sentences)} sentences with voice '{voice}'")
    
    async def generate_audio_events():
        """Generator that yields SSE events with audio chunks."""
        import base64
        
        for i, sentence in enumerate(sentences):
            try:
                # Generate audio for this sentence
                process = subprocess.Popen(
                    [
                        str(PIPER_BIN),
                        "--model", voice_config["model"],
                        "--config", voice_config["config"],
                        "--output-raw",
                        "--length-scale", str(1.0 / request.speed),
                    ],
                    stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                )
                
                audio_raw, _ = process.communicate(input=sentence.encode("utf-8"), timeout=30)
                
                if process.returncode != 0:
                    continue
                
                # Convert to numpy and apply pitch shift
                sample_rate = 22050
                audio_array = np.frombuffer(audio_raw, dtype=np.int16).astype(np.float32) / 32768.0
                
                if request.pitch_shift != 0:
                    shift_factor = 2 ** (request.pitch_shift / 12)
                    new_length = int(len(audio_array) / shift_factor)
                    indices = np.linspace(0, len(audio_array) - 1, new_length)
                    audio_array = np.interp(indices, np.arange(len(audio_array)), audio_array)
                
                # Apply robot effect chain
                if request.robot_effect:
                    logger.info(f"🤖 Applying robot effects to sentence {i}")
                    t = np.arange(len(audio_array)) / sample_rate
                    
                    # 1. High-Pass Filter (400Hz)
                    cutoff_freq = 400
                    rc = 1.0 / (2 * np.pi * cutoff_freq)
                    dt = 1.0 / sample_rate
                    alpha = rc / (rc + dt)
                    
                    filtered = np.zeros_like(audio_array)
                    filtered[0] = audio_array[0]
                    for idx in range(1, len(audio_array)):
                        filtered[idx] = alpha * (filtered[idx-1] + audio_array[idx] - audio_array[idx-1])
                    audio_hp = np.zeros_like(filtered)
                    audio_hp[0] = filtered[0]
                    for idx in range(1, len(filtered)):
                        audio_hp[idx] = alpha * (audio_hp[idx-1] + filtered[idx] - filtered[idx-1])
                    
                    audio_array = audio_hp
                    
                    # 2. Chorus (3 voces, 15ms delay, 50% depth, 2.5Hz LFO, 35% wet)
                    chorus_wet = 0.35
                    chorus_voices = []
                    for voice_idx in range(3):
                        lfo_rate = 2.5 + voice_idx * 0.5
                        delay_ms = 15 + voice_idx * 5
                        depth = 0.5
                        
                        max_delay_samples = int(sample_rate * delay_ms / 1000)
                        lfo = (1 + np.sin(2 * np.pi * lfo_rate * t + voice_idx * np.pi * 2 / 3)) / 2
                        delay_samples = (lfo * max_delay_samples * depth).astype(int) + max_delay_samples // 2
                        
                        voice_out = np.zeros_like(audio_array)
                        for idx in range(len(audio_array)):
                            delay = delay_samples[idx]
                            if idx >= delay:
                                voice_out[idx] = audio_array[idx - delay]
                        chorus_voices.append(voice_out)
                    
                    chorus_mix = sum(chorus_voices) / 3
                    audio_array = audio_array * (1 - chorus_wet) + chorus_mix * chorus_wet
                    
                    # 3. Flanger (1Hz rate, 40% depth, 35% feedback, 30% wet)
                    flanger_rate = 1.0
                    flanger_depth = 0.40
                    flanger_feedback = 0.35
                    flanger_wet = 0.30
                    max_flanger_delay = int(sample_rate * 0.012)  # 12ms max
                    
                    lfo = (1 + np.sin(2 * np.pi * flanger_rate * t)) / 2
                    delay_samples = (lfo * max_flanger_delay * flanger_depth).astype(int)
                    
                    flanged = np.zeros_like(audio_array)
                    feedback_buffer = np.zeros_like(audio_array)
                    for idx in range(len(audio_array)):
                        delay = max(1, delay_samples[idx])
                        if idx >= delay:
                            flanged[idx] = audio_array[idx - delay] + feedback_buffer[idx - delay] * flanger_feedback
                            feedback_buffer[idx] = flanged[idx]
                    
                    audio_array = audio_array * (1 - flanger_wet) + flanged * flanger_wet
                    
                    # 4. Ring Modulation (50Hz carrier, 5% wet)
                    ring_freq = 50
                    ring_wet = 0.05
                    ring_mod = np.sin(2 * np.pi * ring_freq * t)
                    ring_signal = audio_array * ring_mod
                    audio_array = audio_array * (1 - ring_wet) + ring_signal * ring_wet
                    
                    # 5. Room Reverb (12% size, 0.5s decay, 15% wet)
                    reverb_wet = 0.15
                    decay_time = 0.5
                    room_size = 0.12
                    
                    delay_times_ms = [23, 29, 37, 43]
                    reverb_out = np.zeros_like(audio_array)
                    
                    for delay_ms in delay_times_ms:
                        delay_samples_rev = int(sample_rate * delay_ms * room_size / 1000)
                        if delay_samples_rev < 1:
                            delay_samples_rev = 1
                        decay = np.exp(-3 * delay_ms * room_size / 1000 / decay_time)
                        
                        delayed = np.zeros_like(audio_array)
                        for idx in range(delay_samples_rev, len(audio_array)):
                            delayed[idx] = audio_array[idx - delay_samples_rev] * decay
                        reverb_out += delayed
                    
                    reverb_out /= len(delay_times_ms)
                    audio_array = audio_array * (1 - reverb_wet) + reverb_out * reverb_wet
                    
                    # Normalize
                    max_val = np.max(np.abs(audio_array))
                    if max_val > 0.95:
                        audio_array = audio_array * 0.95 / max_val
                
                # Write to WAV buffer
                buffer = io.BytesIO()
                sf.write(buffer, audio_array, sample_rate, format='WAV', subtype='PCM_16')
                buffer.seek(0)
                wav_bytes = buffer.read()
                
                # Encode as base64 and send as SSE
                audio_b64 = base64.b64encode(wav_bytes).decode('utf-8')
                
                event_data = {
                    "type": "audio",
                    "index": i,
                    "total": len(sentences),
                    "data": audio_b64
                }
                
                yield f"data: {json.dumps(event_data)}\n\n"
                
            except Exception as e:
                logger.error(f"Error synthesizing sentence {i}: {e}")
                continue
        
        # Send done event
        yield f"data: {json.dumps({'type': 'done'})}\n\n"
    
    return StreamingResponse(
        generate_audio_events(),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
        }
    )


if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8100)
