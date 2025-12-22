use super::filters::{BiquadCoeffs, BiquadState};
use std::f32::consts::PI;

fn bandpass_coeffs(center: f32, sample_rate: f32, q: f32) -> BiquadCoeffs {
    let omega = 2.0 * PI * center / sample_rate;
    let sin_omega = omega.sin();
    let cos_omega = omega.cos();
    let alpha = sin_omega / (2.0 * q.max(0.01));

    let b0 = alpha;
    let b1 = 0.0;
    let b2 = -alpha;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_omega;
    let a2 = 1.0 - alpha;

    BiquadCoeffs { b0: b0 / a0, b1: b1 / a0, b2: b2 / a0, a1: a1 / a0, a2: a2 / a0 }
}

#[derive(Clone)]
struct Band {
    coeffs: BiquadCoeffs,
    analysis: BiquadState,
    synth: BiquadState,
    env: f32,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum CarrierWave {
    Saw,
    Square,
}

#[allow(dead_code)]
fn saw(phase: f32) -> f32 {
    // phase 0..1
    2.0 * (phase - 0.5)
}

#[allow(dead_code)]
fn square(phase: f32) -> f32 {
    if phase < 0.5 { 1.0 } else { -1.0 }
}

#[allow(dead_code)]
fn autocorr_pitch_hz(frame: &[f32], sample_rate: u32) -> (f32, f32) {
    // Very simple autocorrelation pitch estimate.
    // Returns (f0_hz, confidence 0..1)
    if frame.len() < 256 {
        return (0.0, 0.0);
    }

    let sr = sample_rate as f32;
    let min_hz = 70.0;
    let max_hz = 350.0;

    let min_lag = (sr / max_hz) as usize;
    let max_lag = (sr / min_hz) as usize;

    let max_lag = max_lag.min(frame.len() - 1);

    let mut best_lag = 0usize;
    let mut best = -1e30f32;

    // Energy
    let mut energy = 0.0f32;
    for &x in frame {
        energy += x * x;
    }
    if energy < 1e-6 {
        return (0.0, 0.0);
    }

    for lag in min_lag..=max_lag {
        let mut s = 0.0f32;
        for i in 0..(frame.len() - lag) {
            s += frame[i] * frame[i + lag];
        }
        if s > best {
            best = s;
            best_lag = lag;
        }
    }

    if best_lag == 0 {
        return (0.0, 0.0);
    }

    let f0 = sr / best_lag as f32;
    let conf = (best / energy).clamp(0.0, 1.0);
    (f0, conf)
}

#[allow(dead_code)]
pub fn apply_vocoder(
    audio: &[f32],
    sample_rate: u32,
    bands: usize,
    carrier: CarrierWave,
    depth: f32,
    unvoiced_noise: f32,
    attack_ms: f32,
    release_ms: f32,
    enhance_high: bool,
) -> Vec<f32> {
    if audio.is_empty() || depth < 0.001 {
        return audio.to_vec();
    }

    let sr: f32 = sample_rate as f32;

    let bands = bands.clamp(8, 32);
    let depth = depth.clamp(0.0, 1.0);
    let unvoiced_noise = unvoiced_noise.clamp(0.0, 1.0);

    let attack = (-1.0 / (attack_ms.max(0.1) * 0.001 * sr)).exp();
    let release = (-1.0 / (release_ms.max(0.1) * 0.001 * sr)).exp();

    let f_min: f32 = 120.0;
    let f_max: f32 = 8000.0_f32.min(sr * 0.45);

    let log_min = f_min.ln();
    let log_max = f_max.ln();

    let mut bank: Vec<Band> = (0..bands)
        .map(|i| {
            let t = i as f32 / (bands - 1) as f32;
            let center = (log_min + t * (log_max - log_min)).exp();
            // Wider at low freqs, narrower at highs.
            let q = (1.2 + 0.8 * t).clamp(0.8, 2.2);
            let coeffs = bandpass_coeffs(center, sr, q);
            Band { coeffs, analysis: BiquadState::default(), synth: BiquadState::default(), env: 0.0 }
        })
        .collect();

    let frame_len = (0.046 * sr) as usize; // ~46ms
    let hop = (0.010 * sr) as usize; // 10ms
    let frame_len = frame_len.max(512);
    let hop = hop.max(128);

    let mut phase = 0.0f32;
    let mut f0 = 140.0f32;
    let mut conf = 0.0f32;

    // Simple deterministic noise (LCG)
    let mut rng = 0x1234ABCDu32;
    let mut out = Vec::with_capacity(audio.len());

    let mut next_pitch_at = 0usize;

    for n in 0..audio.len() {
        if n >= next_pitch_at {
            let start = n.saturating_sub(frame_len / 2);
            let end = (start + frame_len).min(audio.len());
            let (nf0, nconf) = autocorr_pitch_hz(&audio[start..end], sample_rate);
            if nf0.is_finite() && nf0 > 50.0 && nf0 < 500.0 {
                f0 = nf0;
                conf = nconf;
            } else {
                conf = 0.0;
            }
            next_pitch_at = n + hop;
        }

        // carrier sample
        phase += f0 / sr;
        if phase >= 1.0 {
            phase -= 1.0;
        }
        let car = match carrier {
            CarrierWave::Saw => saw(phase),
            CarrierWave::Square => square(phase),
        };

        // noise
        rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((rng >> 9) as f32 / ((1u32 << 23) as f32)) * 2.0 - 1.0;

        let mut y = 0.0f32;
        for (i, b) in bank.iter_mut().enumerate() {
            let a = b.analysis.process(audio[n], &b.coeffs);
            let d = a.abs();

            if d > b.env {
                b.env = attack * b.env + (1.0 - attack) * d;
            } else {
                b.env = release * b.env + (1.0 - release) * d;
            }

            let mut env = b.env;
            if enhance_high {
                let t = i as f32 / (bands - 1) as f32;
                env *= 1.0 + 0.25 * t;
            }

            let voicedness = conf;
            let carrier_in = car * voicedness + noise * (1.0 - voicedness) * unvoiced_noise;
            let s = b.synth.process(carrier_in, &b.coeffs);
            y += s * env;
        }

        y /= bands as f32;
        out.push(audio[n] * (1.0 - depth) + y * depth);
    }

    out
}
