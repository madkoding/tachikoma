use std::f32::consts::PI;

pub fn apply_ring_modulation(
    audio: &[f32],
    sample_rate: u32,
    carrier_freq: f32,
    wet: f32,
) -> Vec<f32> {
    if audio.is_empty() || wet < 0.001 {
        return audio.to_vec();
    }

    let sample_rate_f = sample_rate as f32;
    let wet = wet.clamp(0.0, 1.0);

    let mut out = Vec::with_capacity(audio.len());
    for (i, &s) in audio.iter().enumerate() {
        let t = i as f32 / sample_rate_f;
        let carrier = (2.0 * PI * carrier_freq * t).sin();
        let modulated = s * carrier;
        out.push(s * (1.0 - wet) + modulated * wet);
    }

    out
}
