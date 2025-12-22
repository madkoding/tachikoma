use super::SAMPLE_RATE;
use super::pitch_shift::apply_pitch_shift;
use super::reverb::apply_reverb;
use super::limiter::apply_limiter;
use super::chorus::apply_chorus;

fn normalize_peak_in_place(audio: &mut [f32], max_level: f32) {
    let max_val = audio.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    if max_val > max_level {
        let scale = max_level / max_val;
        for s in audio.iter_mut() {
            *s *= scale;
        }
    }
}

/// ESTRATEGIA DOBLE CAPA:
/// - Capa 1: Voz original 100% sin filtros
/// - Capa 2: Pitch +1, chorus, reverb CON 30ms de delay (voces paralelas)
#[allow(clippy::too_many_arguments)]
pub fn apply_robot_effect_chain(
    audio: &[f32],
    _highpass_cutoff: f32,
    _chorus_wet: f32,
    _chorus_voices: usize,
    _flanger_rate: f32,
    _flanger_depth: f32,
    _flanger_feedback: f32,
    _flanger_wet: f32,
    _ring_freq: f32,
    _ring_wet: f32,
    _reverb_wet: f32,
    _reverb_decay: f32,
    _reverb_room_size: f32,
) -> Vec<f32> {
    if audio.is_empty() {
        return Vec::new();
    }

    // =========================================================================
    // CAPA 1: Voz original 100% SIN FILTROS
    // =========================================================================
    let dry = audio;

    // =========================================================================
    // CAPA 2: Pitch +6, chorus, reverb CON 30ms de delay
    // =========================================================================
    
    // Pitch shift +6 semitonos (media octava más agudo)
    let mut wet = apply_pitch_shift(audio, 6.0);

    // Chorus suave
    wet = apply_chorus(&wet, SAMPLE_RATE, 0.25, 2);

    // Reverb suave
    wet = apply_reverb(&wet, SAMPLE_RATE, 0.2, 0.5, 0.25);

    // =========================================================================
    // MEZCLA: dry (sin delay) + wet (con 30ms de delay)
    // =========================================================================
    
    // 30ms de delay en samples: 22050 * 0.030 = 661 samples
    let delay_samples = (SAMPLE_RATE as f32 * 0.030) as usize; // 30ms
    
    // La salida debe ser lo suficientemente larga para ambas
    let out_len = dry.len().max(wet.len() + delay_samples);
    let mut out = vec![0.0f32; out_len];

    // Agregar dry (voz original) desde el inicio
    for (i, &dry_sample) in dry.iter().enumerate() {
        out[i] += dry_sample * 0.55; // 55% voz original
    }
    
    // Agregar wet (voz procesada) con delay de 30ms
    for (i, &wet_sample) in wet.iter().enumerate() {
        let out_idx = i + delay_samples;
        if out_idx < out_len {
            out[out_idx] += wet_sample * 0.45; // 45% voz procesada con delay
        }
    }

    // Limiter final para evitar clipping
    out = apply_limiter(&out, -0.5, 80.0, SAMPLE_RATE);

    normalize_peak_in_place(&mut out, 0.95);
    out
}
