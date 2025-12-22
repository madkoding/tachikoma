FROM python:3.11-slim

# Install system dependencies (including libsndfile for soundfile)
RUN apt-get update && apt-get install -y --no-install-recommends \
    wget \
    tar \
    libsndfile1 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Download and install Piper TTS
RUN wget -q https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_linux_x86_64.tar.gz \
    && tar -xzf piper_linux_x86_64.tar.gz \
    && rm piper_linux_x86_64.tar.gz \
    && chmod +x /app/piper/piper

# Create models directory and download Spanish female voice (Cortana)
RUN mkdir -p /app/models \
    && cd /app/models \
    && wget -q https://huggingface.co/rhasspy/piper-voices/resolve/main/es/es_MX/claude/high/es_MX-claude-high.onnx \
    && wget -q https://huggingface.co/rhasspy/piper-voices/resolve/main/es/es_MX/claude/high/es_MX-claude-high.onnx.json

# Install Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY main.py .

# Expose port
EXPOSE 8100

# Run the service
CMD ["uvicorn", "main:app", "--host", "0.0.0.0", "--port", "8100"]
