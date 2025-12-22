use std::f32::consts::PI;

/// Pre-computed constants for chorus LFO
const TWO_PI: f32 = 2.0 * PI;

pub fn apply_chorus(audio: &[f32], sample_rate: u32, wet: f32, num_voices: usize) -> Vec<f32> {
    if audio.is_empty() || wet < 0.001 || num_voices == 0 {
        return audio.to_vec();
    }

    let sample_rate_f = sample_rate as f32;
    let inv_sample_rate = 1.0 / sample_rate_f;  // Pre-compute inverse to avoid division in loop
    let len = audio.len();
    let mut output = vec![0.0f32; len];
    let inv_num_voices = 1.0 / num_voices as f32;  // Pre-compute for phase calculation
    let ms_to_samples = sample_rate_f * 0.001;  // Pre-compute conversion factor

    for voice_idx in 0..num_voices {
        let voice_idx_f = voice_idx as f32;
        let lfo_rate = 0.35 + voice_idx_f * 0.15;
        let base_delay_ms = 18.0 + voice_idx_f * 4.0;
        let phase_offset = voice_idx_f * TWO_PI * inv_num_voices;

        let base_delay_samples = (ms_to_samples * base_delay_ms) as usize;
        let depth_samples = ms_to_samples * 4.5;  // depth_ms = 4.5
        
        // Pre-compute LFO angular frequency multiplied by inverse sample rate
        let lfo_freq = TWO_PI * lfo_rate * inv_sample_rate;

        for i in 0..len {
            // Optimized: multiply by pre-computed frequency instead of divide
            let lfo = (lfo_freq * i as f32 + phase_offset).sin();
            let delay = base_delay_samples as f32 + lfo * depth_samples;
            let delay_int = delay as usize;  // Faster than floor() for positive values
            let delay_frac = delay - delay_int as f32;

            if i > delay_int {
                let idx = i - delay_int;
                let s1 = audio[idx];
                let s2 = audio[idx - 1];
                // Optimized interpolation: s2 + frac * (s1 - s2) = s1*(1-frac) + s2*frac
                output[i] += s2 + delay_frac * (s1 - s2);
            }
        }
    }

    let wet = wet.clamp(0.0, 1.0);
    let dry = 1.0 - wet;
    let wet_norm = wet * inv_num_voices;  // Combine normalization with wet mix

    for i in 0..len {
        output[i] = audio[i] * dry + output[i] * wet_norm;
    }

    output
}
