/// Soft clip / tape-ish saturation using tanh.
/// drive: 0.0..1.0 (maps to internal gain)
pub fn apply_saturation(audio: &[f32], drive: f32, mix: f32) -> Vec<f32> {
    if audio.is_empty() || mix < 0.001 || drive < 0.001 {
        return audio.to_vec();
    }

    let mix = mix.clamp(0.0, 1.0);
    let drive = drive.clamp(0.0, 1.0);

    // Map 0..1 to a gentle but audible range.
    let g = 1.0 + drive * 8.0;
    let norm = (g).tanh();

    let mut out = Vec::with_capacity(audio.len());
    for &x in audio {
        let y = (x * g).tanh() / norm;
        out.push(x * (1.0 - mix) + y * mix);
    }
    out
}
