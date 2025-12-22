use super::filters::{bandpass_process_in_place};

#[allow(dead_code)]
fn db_to_lin(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

/// Simple de-esser: extracts 4-8k-ish band, applies gain reduction, mixes back.
/// reduction_db is positive (e.g. 6.0 means up to -6 dB).
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn apply_deesser(
    audio: &[f32],
    center_hz: f32,
    q: f32,
    reduction_db: f32,
    threshold_db: f32,
    attack_ms: f32,
    release_ms: f32,
    mix: f32,
    sample_rate: u32,
) -> Vec<f32> {
    if audio.is_empty() || mix <= 0.001 || reduction_db <= 0.01 {
        return audio.to_vec();
    }

    let mix = mix.clamp(0.0, 1.0);
    let sr = sample_rate as f32;

    let attack = (-1.0 / (attack_ms.max(0.1) * 0.001 * sr)).exp();
    let release = (-1.0 / (release_ms.max(0.1) * 0.001 * sr)).exp();

    let threshold = db_to_lin(threshold_db);
    let max_reduction = db_to_lin(-reduction_db.abs());

    let mut band = audio.to_vec();
    bandpass_process_in_place(&mut band, center_hz, q, sample_rate);

    let mut env = 0.0f32;
    let mut out = Vec::with_capacity(audio.len());

    for (dry, s) in audio.iter().zip(band.iter()) {
        let d = s.abs();
        if d > env {
            env = attack * env + (1.0 - attack) * d;
        } else {
            env = release * env + (1.0 - release) * d;
        }

        let mut g = 1.0f32;
        if env > threshold {
            // Linear interpolation of reduction above threshold
            let over = (env / threshold).min(10.0);
            let t = ((over - 1.0) / 9.0).clamp(0.0, 1.0);
            g = 1.0 + t * (max_reduction - 1.0);
        }

        let processed = *dry - (*s) + (*s) * g;
        out.push(*dry * (1.0 - mix) + processed * mix);
    }

    out
}
