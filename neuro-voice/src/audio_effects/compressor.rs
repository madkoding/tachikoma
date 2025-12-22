fn db_to_lin(db: f32) -> f32 {
    10.0_f32.powf(db / 20.0)
}

/// Simple feed-forward compressor (mono) with soft knee-ish behavior.
pub fn apply_compressor(
    audio: &[f32],
    threshold_db: f32,
    ratio: f32,
    attack_ms: f32,
    release_ms: f32,
    makeup_db: f32,
    sample_rate: u32,
) -> Vec<f32> {
    if audio.is_empty() {
        return Vec::new();
    }

    let sr = sample_rate as f32;
    let threshold = db_to_lin(threshold_db);
    let ratio = ratio.max(1.0);

    let attack = (-1.0 / (attack_ms.max(0.1) * 0.001 * sr)).exp();
    let release = (-1.0 / (release_ms.max(0.1) * 0.001 * sr)).exp();

    let mut env = 0.0f32;
    let makeup = db_to_lin(makeup_db);

    let mut out = Vec::with_capacity(audio.len());

    for &x in audio {
        let detector = x.abs();
        if detector > env {
            env = attack * env + (1.0 - attack) * detector;
        } else {
            env = release * env + (1.0 - release) * detector;
        }

        let mut gain = 1.0f32;
        if env > threshold {
            // Gain computer in linear domain
            // desired_out = threshold * (env/threshold)^(1/ratio)
            let desired = threshold * (env / threshold).powf(1.0 / ratio);
            gain = (desired / env).clamp(0.0, 1.0);
        }

        out.push(x * gain * makeup);
    }

    out
}
