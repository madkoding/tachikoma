use super::SAMPLE_RATE;

/// Pitch shift simple y efectivo usando resampling
/// Sube el pitch sin cambiar la duración significativamente
pub fn apply_pitch_shift(audio: &[f32], semitones: f32) -> Vec<f32> {
    if audio.is_empty() || semitones.abs() < 0.01 {
        return audio.to_vec();
    }

    // Factor de pitch: 2^(semitones/12)
    // +1 semitono = 1.059x, +4 semitonos = 1.26x, +12 = 2x
    let pitch_factor = 2.0_f32.powf(semitones / 12.0);
    
    // Para subir el pitch, necesitamos MENOS samples (reproducir más rápido)
    // Luego estiramos de vuelta a la longitud original
    let resampled_len = (audio.len() as f32 / pitch_factor) as usize;
    let resampled_len = resampled_len.max(1);
    
    // Paso 1: Resample a longitud más corta (sube el pitch)
    let mut resampled = Vec::with_capacity(resampled_len);
    for i in 0..resampled_len {
        let src_pos = i as f32 * pitch_factor;
        let idx = src_pos.floor() as usize;
        let frac = src_pos - idx as f32;
        
        let s0 = audio.get(idx).copied().unwrap_or(0.0);
        let s1 = audio.get(idx + 1).copied().unwrap_or(s0);
        
        // Interpolación lineal
        resampled.push(s0 * (1.0 - frac) + s1 * frac);
    }
    
    // Paso 2: Estirar de vuelta usando PSOLA simplificado
    // Esto mantiene el pitch alto pero restaura la duración
    psola_stretch(&resampled, audio.len(), SAMPLE_RATE)
}

/// PSOLA simplificado para time-stretch sin cambiar pitch
fn psola_stretch(audio: &[f32], target_len: usize, sample_rate: u32) -> Vec<f32> {
    if audio.is_empty() || target_len == 0 {
        return Vec::new();
    }
    
    // Si son muy similares, solo resample lineal
    let len_ratio = target_len as f32 / audio.len() as f32;
    if (len_ratio - 1.0).abs() < 0.05 {
        return linear_resample(audio, target_len);
    }
    
    let sr = sample_rate as f32;
    
    // Tamaño de ventana ~25ms (bueno para voz)
    let window_size = (0.025 * sr) as usize;
    let window_size = window_size.max(128).min(audio.len());
    
    // Hop de análisis y síntesis
    let ana_hop = window_size / 4;
    let syn_hop = ((ana_hop as f32) * len_ratio) as usize;
    let syn_hop = syn_hop.max(1);
    
    // Ventana Hann
    let window: Vec<f32> = (0..window_size)
        .map(|i| 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (window_size - 1) as f32).cos()))
        .collect();
    
    let mut out = vec![0.0f32; target_len + window_size];
    let mut sum_win = vec![0.0f32; target_len + window_size];
    
    let mut ana_pos = 0usize;
    let mut syn_pos = 0usize;
    
    while ana_pos + window_size <= audio.len() && syn_pos + window_size <= out.len() {
        // Overlap-add con ventana
        for i in 0..window_size {
            let w = window[i];
            out[syn_pos + i] += audio[ana_pos + i] * w;
            sum_win[syn_pos + i] += w * w;
        }
        
        ana_pos += ana_hop;
        syn_pos += syn_hop;
    }
    
    // Normalizar
    for i in 0..out.len() {
        if sum_win[i] > 0.001 {
            out[i] /= sum_win[i];
        }
    }
    
    out.truncate(target_len);
    out
}

/// Resample lineal simple
fn linear_resample(audio: &[f32], target_len: usize) -> Vec<f32> {
    if audio.is_empty() || target_len == 0 {
        return Vec::new();
    }
    if audio.len() == target_len {
        return audio.to_vec();
    }

    let mut out = Vec::with_capacity(target_len);
    let ratio = (audio.len() - 1) as f64 / (target_len - 1).max(1) as f64;
    
    for i in 0..target_len {
        let pos = i as f64 * ratio;
        let idx = pos.floor() as usize;
        let frac = (pos - idx as f64) as f32;
        let next = (idx + 1).min(audio.len() - 1);
        out.push(audio[idx] * (1.0 - frac) + audio[next] * frac);
    }
    out
}
