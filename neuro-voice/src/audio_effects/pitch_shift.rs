use super::SAMPLE_RATE;

/// Constante pre-calculada: ln(2) para conversión de semitonos
const LN_2: f32 = 0.693147180559945;
const INV_12: f32 = 1.0 / 12.0;

/// Calcula 2^x usando exp para mejor rendimiento que powf
/// 2^x = e^(x * ln(2))
#[inline(always)]
fn fast_pow2(x: f32) -> f32 {
    (x * LN_2).exp()
}

/// Pitch shift simple y efectivo usando resampling
/// Sube el pitch sin cambiar la duración significativamente
pub fn apply_pitch_shift(audio: &[f32], semitones: f32) -> Vec<f32> {
    if audio.is_empty() || semitones.abs() < 0.01 {
        return audio.to_vec();
    }

    // Factor de pitch: 2^(semitones/12) usando fast_pow2
    // +1 semitono = 1.059x, +4 semitonos = 1.26x, +12 = 2x
    let pitch_factor = fast_pow2(semitones * INV_12);
    
    // Para subir el pitch, necesitamos MENOS samples (reproducir más rápido)
    // Luego estiramos de vuelta a la longitud original
    let resampled_len = (audio.len() as f32 / pitch_factor) as usize;
    let resampled_len = resampled_len.max(1);
    let audio_len = audio.len();
    
    // Paso 1: Resample a longitud más corta (sube el pitch)
    let mut resampled = Vec::with_capacity(resampled_len);
    for i in 0..resampled_len {
        let src_pos = i as f32 * pitch_factor;
        let idx = src_pos as usize;  // Más rápido que floor() para valores positivos
        let frac = src_pos - idx as f32;
        
        // Acceso directo con bounds check optimizado
        let s0 = if idx < audio_len { audio[idx] } else { 0.0 };
        let s1 = if idx + 1 < audio_len { audio[idx + 1] } else { s0 };
        
        // Interpolación lineal optimizada: s0 + frac * (s1 - s0)
        resampled.push(s0 + frac * (s1 - s0));
    }
    
    // Paso 2: Estirar de vuelta usando PSOLA simplificado
    // Esto mantiene el pitch alto pero restaura la duración
    psola_stretch(&resampled, audio.len(), SAMPLE_RATE)
}

/// Pre-compute Hann window coefficients
/// w[i] = 0.5 * (1 - cos(2*PI*i/(N-1)))
#[inline]
fn compute_hann_window(size: usize) -> Vec<f32> {
    let inv_size_minus_1 = std::f32::consts::PI * 2.0 / (size - 1) as f32;
    (0..size)
        .map(|i| 0.5 * (1.0 - (i as f32 * inv_size_minus_1).cos()))
        .collect()
}

/// PSOLA simplificado para time-stretch sin cambiar pitch
fn psola_stretch(audio: &[f32], target_len: usize, sample_rate: u32) -> Vec<f32> {
    if audio.is_empty() || target_len == 0 {
        return Vec::new();
    }
    
    let audio_len = audio.len();
    
    // Si son muy similares, solo resample lineal
    let len_ratio = target_len as f32 / audio_len as f32;
    if (len_ratio - 1.0).abs() < 0.05 {
        return linear_resample(audio, target_len);
    }
    
    // Tamaño de ventana ~25ms (bueno para voz)
    let window_size = ((0.025 * sample_rate as f32) as usize).clamp(128, audio_len);
    
    // Hop de análisis y síntesis
    let ana_hop = window_size >> 2;  // Más rápido que / 4
    let syn_hop = ((ana_hop as f32) * len_ratio) as usize;
    let syn_hop = syn_hop.max(1);
    
    // Ventana Hann pre-calculada
    let window = compute_hann_window(window_size);
    
    // Pre-calcular w^2 para evitar multiplicación en el loop
    let window_sq: Vec<f32> = window.iter().map(|&w| w * w).collect();
    
    let out_len = target_len + window_size;
    let mut out = vec![0.0f32; out_len];
    let mut sum_win = vec![0.0f32; out_len];
    
    let mut ana_pos = 0usize;
    let mut syn_pos = 0usize;
    
    while ana_pos + window_size <= audio_len && syn_pos + window_size <= out_len {
        // Overlap-add con ventana - acceso directo sin bounds check
        let audio_slice = &audio[ana_pos..ana_pos + window_size];
        let out_slice = &mut out[syn_pos..syn_pos + window_size];
        let sum_slice = &mut sum_win[syn_pos..syn_pos + window_size];
        
        for i in 0..window_size {
            out_slice[i] += audio_slice[i] * window[i];
            sum_slice[i] += window_sq[i];
        }
        
        ana_pos += ana_hop;
        syn_pos += syn_hop;
    }
    
    // Normalizar con umbral de reciprocidad
    for i in 0..out_len {
        if sum_win[i] > 0.001 {
            out[i] *= 1.0 / sum_win[i];  // Multiplicar por recíproco puede ser más rápido
        }
    }
    
    out.truncate(target_len);
    out
}

/// Resample lineal simple - optimizado
fn linear_resample(audio: &[f32], target_len: usize) -> Vec<f32> {
    if audio.is_empty() || target_len == 0 {
        return Vec::new();
    }
    let audio_len = audio.len();
    if audio_len == target_len {
        return audio.to_vec();
    }

    let mut out = Vec::with_capacity(target_len);
    let last_audio_idx = audio_len - 1;
    // Usar f32 en lugar de f64 para mejor rendimiento en operaciones repetidas
    let ratio = (audio_len - 1) as f32 / (target_len - 1).max(1) as f32;
    
    for i in 0..target_len {
        let pos = i as f32 * ratio;
        let idx = pos as usize;  // Más rápido que floor() para valores positivos
        let frac = pos - idx as f32;
        let next = (idx + 1).min(last_audio_idx);
        // Interpolación optimizada: s0 + frac * (s1 - s0)
        out.push(audio[idx] + frac * (audio[next] - audio[idx]));
    }
    out
}
