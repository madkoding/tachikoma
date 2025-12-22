use super::filters::{peaking_eq_process_in_place};

/// Approximate formant shift using peaking EQ boosts at shifted formant regions.
///
/// NOTE: This is not a true LPC/formant-preserving shifter; it is a pragmatic
/// approximation that tends to produce a more "infantil/robótico" character,
/// especially when followed by a vocoder.
pub fn apply_formant_shift_approx(
    audio: &[f32],
    shift_pct: f32,
    sample_rate: u32,
) -> Vec<f32> {
    if audio.is_empty() {
        return Vec::new();
    }

    let shift = 1.0 + shift_pct.clamp(0.0, 0.5);

    let f1 = 500.0 * shift; // within ~400-600 base
    let f2 = 2000.0 * shift; // within ~1800-2200 base

    let mut out = audio.to_vec();

    // Gentle boosts to mimic formant raise
    peaking_eq_process_in_place(&mut out, f1, 1.2, 2.5, sample_rate);
    peaking_eq_process_in_place(&mut out, f2, 1.4, 3.0, sample_rate);

    out
}
