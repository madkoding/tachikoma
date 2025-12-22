pub struct CombFilter {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
}

impl CombFilter {
    fn new(delay_samples: usize, feedback: f32) -> Self {
        Self { buffer: vec![0.0; delay_samples.max(1)], index: 0, feedback }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.buffer[self.index];
        self.buffer[self.index] = input + output * self.feedback;
        self.index = (self.index + 1) % self.buffer.len();
        output
    }
}

pub struct AllPassFilter {
    buffer: Vec<f32>,
    index: usize,
    feedback: f32,
}

impl AllPassFilter {
    fn new(delay_samples: usize, feedback: f32) -> Self {
        Self { buffer: vec![0.0; delay_samples.max(1)], index: 0, feedback }
    }

    fn process(&mut self, input: f32) -> f32 {
        let buffered = self.buffer[self.index];
        let output = -input + buffered;
        self.buffer[self.index] = input + buffered * self.feedback;
        self.index = (self.index + 1) % self.buffer.len();
        output
    }
}

pub fn apply_reverb(audio: &[f32], sample_rate: u32, room_size: f32, decay_time: f32, wet: f32) -> Vec<f32> {
    if audio.is_empty() || wet < 0.001 {
        return audio.to_vec();
    }

    let sample_rate_f = sample_rate as f32;
    let wet = wet.clamp(0.0, 1.0);
    let dry = 1.0 - wet;  // Pre-calcular dry

    let room = room_size.clamp(0.1, 1.0);
    let _decay = decay_time.clamp(0.05, 5.0);

    // Pragmatic mapping: keep it stable and small
    let base_feedback = 0.78 * room;
    
    // Pre-calcular factor de conversión ms->samples
    let ms_to_samples = room * sample_rate_f * 0.001;

    let comb_delays_ms = [29.7, 37.1, 41.1, 43.7];
    let mut combs: Vec<CombFilter> = comb_delays_ms
        .iter()
        .map(|&d| {
            let delay = (d * ms_to_samples) as usize;
            CombFilter::new(delay, base_feedback)
        })
        .collect();
    
    // Pre-calcular inversa del número de combs para evitar división en el loop
    let inv_num_combs = 1.0 / combs.len() as f32;

    let allpass_delays_ms = [5.0, 1.7];
    let mut allpasses: Vec<AllPassFilter> = allpass_delays_ms
        .iter()
        .map(|&d| {
            let delay = (d * ms_to_samples) as usize;
            AllPassFilter::new(delay, 0.5)
        })
        .collect();

    let mut out = Vec::with_capacity(audio.len());

    for &s in audio {
        let mut comb_sum = 0.0f32;
        for c in &mut combs {
            comb_sum += c.process(s);
        }
        // Multiplicar por inversa en lugar de dividir
        comb_sum *= inv_num_combs;

        let mut diff = comb_sum;
        for ap in &mut allpasses {
            diff = ap.process(diff);
        }

        out.push(s * dry + diff * wet);
    }

    out
}
