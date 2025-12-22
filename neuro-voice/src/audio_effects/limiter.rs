fn db_to_lin(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

/// Simple peak limiter with release smoothing (no lookahead).
pub fn apply_limiter(audio: &[f32], ceiling_db: f32, release_ms: f32, sample_rate: u32) -> Vec<f32> {
    if audio.is_empty() {
        return Vec::new();
    }

    let ceiling = db_to_lin(ceiling_db);
    let sr = sample_rate as f32;
    let release = (-1.0 / (release_ms.max(0.1) * 0.001 * sr)).exp();

    let mut gain = 1.0f32;
    let mut out = Vec::with_capacity(audio.len());

    for &x in audio {
        let peak = x.abs();
        if peak * gain > ceiling {
            gain = (ceiling / peak).min(gain);
        } else {
            // recover slowly
            gain = release * gain + (1.0 - release) * 1.0;
        }
        out.push(x * gain);
    }

    out
}
