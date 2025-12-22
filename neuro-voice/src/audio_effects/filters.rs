use std::f32::consts::PI;

#[derive(Clone, Copy)]
pub struct BiquadCoeffs {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
}

#[derive(Clone, Copy, Default)]
pub struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
}

#[allow(dead_code)]
impl BiquadState {
    pub fn process(&mut self, x0: f32, c: &BiquadCoeffs) -> f32 {
        let y0 = c.b0 * x0 + c.b1 * self.x1 + c.b2 * self.x2 - c.a1 * self.y1 - c.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x0;
        self.y2 = self.y1;
        self.y1 = y0;
        y0
    }
}

#[allow(dead_code)]
fn highpass_coeffs(cutoff: f32, sample_rate: f32, q: f32) -> BiquadCoeffs {
    let omega = 2.0 * PI * cutoff / sample_rate;
    let sin_omega = omega.sin();
    let cos_omega = omega.cos();
    let alpha = sin_omega / (2.0 * q);

    let b0 = (1.0 + cos_omega) / 2.0;
    let b1 = -(1.0 + cos_omega);
    let b2 = (1.0 + cos_omega) / 2.0;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_omega;
    let a2 = 1.0 - alpha;

    BiquadCoeffs {
        b0: b0 / a0,
        b1: b1 / a0,
        b2: b2 / a0,
        a1: a1 / a0,
        a2: a2 / a0,
    }
}

#[allow(dead_code)]
fn bandpass_coeffs(center: f32, sample_rate: f32, q: f32) -> BiquadCoeffs {
    let omega = 2.0 * PI * center / sample_rate;
    let sin_omega = omega.sin();
    let cos_omega = omega.cos();
    let alpha = sin_omega / (2.0 * q.max(0.01));

    // Constant skirt gain, peak gain = Q
    let b0 = alpha;
    let b1 = 0.0;
    let b2 = -alpha;
    let a0 = 1.0 + alpha;
    let a1 = -2.0 * cos_omega;
    let a2 = 1.0 - alpha;

    BiquadCoeffs {
        b0: b0 / a0,
        b1: b1 / a0,
        b2: b2 / a0,
        a1: a1 / a0,
        a2: a2 / a0,
    }
}

#[allow(dead_code)]
fn peaking_eq_coeffs(center: f32, sample_rate: f32, q: f32, gain_db: f32) -> BiquadCoeffs {
    // RBJ peaking EQ
    let a = 10.0_f32.powf(gain_db / 40.0);
    let omega = 2.0 * PI * center / sample_rate;
    let sin_omega = omega.sin();
    let cos_omega = omega.cos();
    let alpha = sin_omega / (2.0 * q.max(0.01));

    let b0 = 1.0 + alpha * a;
    let b1 = -2.0 * cos_omega;
    let b2 = 1.0 - alpha * a;
    let a0 = 1.0 + alpha / a;
    let a1 = -2.0 * cos_omega;
    let a2 = 1.0 - alpha / a;

    BiquadCoeffs {
        b0: b0 / a0,
        b1: b1 / a0,
        b2: b2 / a0,
        a1: a1 / a0,
        a2: a2 / a0,
    }
}
#[allow(dead_code)]#[allow(dead_code)]
pub fn apply_highpass_filter(audio: &[f32], cutoff_freq: f32, sample_rate: u32) -> Vec<f32> {
    if audio.is_empty() || cutoff_freq < 1.0 {
        return audio.to_vec();
    }

    let sr = sample_rate as f32;
    let coeffs = highpass_coeffs(cutoff_freq, sr, 0.707);

    // 2 cascaded biquads => ~24 dB/oct
    let mut state1 = BiquadState::default();
    let mut state2 = BiquadState::default();

    let mut out = Vec::with_capacity(audio.len());
    for &x in audio {
        let y1 = state1.process(x, &coeffs);
        let y2 = state2.process(y1, &coeffs);
        out.push(y2);
    }
    out
}

/// Gentle 1-pole low-pass (≈6 dB/oct) RC filter.
#[allow(dead_code)]
pub fn apply_lowpass_filter_rc(audio: &[f32], cutoff_hz: f32, sample_rate: u32) -> Vec<f32> {
    if audio.is_empty() || cutoff_hz < 1.0 {
        return audio.to_vec();
    }

    let sr = sample_rate as f32;
    let dt = 1.0 / sr;
    let rc = 1.0 / (2.0 * PI * cutoff_hz);
    let alpha = dt / (rc + dt);

    let mut y = 0.0f32;
    let mut out = Vec::with_capacity(audio.len());
    for &x in audio {
        y = y + alpha * (x - y);
        out.push(y);
    }
    out
}

#[allow(dead_code)]
pub fn apply_resonant_bandpass(audio: &[f32], center_hz: f32, q: f32, gain_db: f32, sample_rate: u32) -> Vec<f32> {
    if audio.is_empty() {
        return Vec::new();
    }
    let sr = sample_rate as f32;

    let bp = bandpass_coeffs(center_hz, sr, q);
    let peak = peaking_eq_coeffs(center_hz, sr, q, gain_db);

    let mut bp_state = BiquadState::default();
    let mut peak_state = BiquadState::default();

    let mut out = Vec::with_capacity(audio.len());
    for &x in audio {
        let y = bp_state.process(x, &bp);
        let y = peak_state.process(y, &peak);
        out.push(y);
    }
    out
}

#[allow(dead_code)]
pub fn peaking_eq_process_in_place(audio: &mut [f32], center_hz: f32, q: f32, gain_db: f32, sample_rate: u32) {
    if audio.is_empty() {
        return;
    }
    let sr = sample_rate as f32;
    let c = peaking_eq_coeffs(center_hz, sr, q, gain_db);
    let mut st = BiquadState::default();
    for x in audio.iter_mut() {
        *x = st.process(*x, &c);
    }
}

#[allow(dead_code)]
pub fn bandpass_process_in_place(audio: &mut [f32], center_hz: f32, q: f32, sample_rate: u32) {
    if audio.is_empty() {
        return;
    }
    let sr = sample_rate as f32;
    let c = bandpass_coeffs(center_hz, sr, q);
    let mut st = BiquadState::default();
    for x in audio.iter_mut() {
        *x = st.process(*x, &c);
    }
}
