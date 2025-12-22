use std::f32::consts::PI;

#[allow(dead_code)]
pub fn apply_flanger(
    audio: &[f32],
    sample_rate: u32,
    rate: f32,
    depth: f32,
    feedback: f32,
    wet: f32,
) -> Vec<f32> {
    if audio.is_empty() || wet < 0.001 {
        return audio.to_vec();
    }

    let sample_rate_f = sample_rate as f32;
    let len = audio.len();

    let min_delay_ms = 0.1;
    let max_delay_ms = 7.0;
    let min_delay_samples = (sample_rate_f * min_delay_ms / 1000.0) as usize;
    let max_delay_samples = (sample_rate_f * max_delay_ms / 1000.0) as usize;
    let delay_range = (max_delay_samples - min_delay_samples) as f32;

    let mut delay_buffer = vec![0.0f32; max_delay_samples + 10];
    let mut buffer_pos = 0usize;
    let mut output = Vec::with_capacity(len);

    let feedback = feedback.clamp(0.0, 0.95);

    for (i, _) in audio.iter().enumerate().take(len) {
        let t = i as f32 / sample_rate_f;
        let lfo = (2.0 * PI * rate * t).sin() * 0.5 + 0.5;
        let delay_samples = min_delay_samples as f32 + lfo * delay_range * depth.clamp(0.0, 1.0);

        let delay_int = delay_samples.floor() as usize;
        let delay_frac = delay_samples - delay_int as f32;

        let read_pos1 = (buffer_pos + delay_buffer.len() - delay_int) % delay_buffer.len();
        let read_pos2 = (buffer_pos + delay_buffer.len() - delay_int - 1) % delay_buffer.len();
        let delayed = delay_buffer[read_pos1] * (1.0 - delay_frac) + delay_buffer[read_pos2] * delay_frac;

        delay_buffer[buffer_pos] = audio[i] + delayed * feedback;

        let wet = wet.clamp(0.0, 1.0);
        output.push(audio[i] * (1.0 - wet) + delayed * wet);

        buffer_pos = (buffer_pos + 1) % delay_buffer.len();
    }

    output
}
