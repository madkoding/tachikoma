use std::f32::consts::PI;

pub fn apply_chorus(audio: &[f32], sample_rate: u32, wet: f32, num_voices: usize) -> Vec<f32> {
    if audio.is_empty() || wet < 0.001 || num_voices == 0 {
        return audio.to_vec();
    }

    let sample_rate_f = sample_rate as f32;
    let len = audio.len();
    let mut output = vec![0.0f32; len];

    for voice_idx in 0..num_voices {
        let lfo_rate = 0.35 + voice_idx as f32 * 0.15; // 0.35Hz..0.65Hz typical
        let base_delay_ms = 18.0 + voice_idx as f32 * 4.0;
        let depth_ms = 4.5;
        let phase_offset = voice_idx as f32 * 2.0 * PI / num_voices as f32;

        let base_delay_samples = (sample_rate_f * base_delay_ms / 1000.0) as usize;
        let depth_samples = sample_rate_f * depth_ms / 1000.0;

        for i in 0..len {
            let t = i as f32 / sample_rate_f;
            let lfo = (2.0 * PI * lfo_rate * t + phase_offset).sin();
            let delay = base_delay_samples as f32 + lfo * depth_samples;
            let delay_int = delay.floor() as usize;
            let delay_frac = delay - delay_int as f32;

            if i > delay_int {
                let s1 = audio[i - delay_int];
                let s2 = audio[i - delay_int - 1];
                let delayed = s1 * (1.0 - delay_frac) + s2 * delay_frac;
                output[i] += delayed;
            }
        }
    }

    let norm = 1.0 / num_voices as f32;
    let wet = wet.clamp(0.0, 1.0);

    for i in 0..len {
        output[i] = audio[i] * (1.0 - wet) + output[i] * norm * wet;
    }

    output
}
