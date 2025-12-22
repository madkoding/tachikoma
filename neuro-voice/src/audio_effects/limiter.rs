/// Constante para conversión de dB: ln(10) / 20
const DB_TO_LIN_FACTOR: f32 = 0.11512925464970229;

/// Convierte dB a lineal usando e^(db * ln(10)/20) - más rápido que powf
#[inline(always)]
fn db_to_lin(db: f32) -> f32 {
    (db * DB_TO_LIN_FACTOR).exp()
}

/// Simple peak limiter with release smoothing (no lookahead).
pub fn apply_limiter(audio: &[f32], ceiling_db: f32, release_ms: f32, sample_rate: u32) -> Vec<f32> {
    if audio.is_empty() {
        return Vec::new();
    }

    let ceiling = db_to_lin(ceiling_db);
    // Pre-calcular coeficiente de release una sola vez
    let release_time = release_ms.max(0.1) * 0.001 * sample_rate as f32;
    let release = (-1.0 / release_time).exp();
    let one_minus_release = 1.0 - release;  // Pre-calcular para el loop

    let mut gain = 1.0f32;
    let mut out = Vec::with_capacity(audio.len());

    for &x in audio {
        let peak = x.abs();
        let current_output = peak * gain;
        if current_output > ceiling {
            // División solo cuando es necesario
            gain = ceiling / peak;
        } else {
            // recover slowly - optimizado con valor pre-calculado
            gain = release * gain + one_minus_release;
        }
        out.push(x * gain);
    }

    out
}
