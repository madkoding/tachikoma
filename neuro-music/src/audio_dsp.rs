//! Audio DSP processing: equalizer, enhancement, spectrum analysis

use rustfft::{num_complex::Complex, FftPlanner};
use std::f32::consts::PI;

/// 16-band equalizer frequencies (in Hz)
pub const EQUALIZER_FREQUENCIES: [f32; 16] = [
    32.0,    // Sub-bass
    64.0,    // Bass
    125.0,   // Low bass
    250.0,   // Low mid
    500.0,   // Mid
    1000.0,  // Mid
    2000.0,  // Upper mid
    4000.0,  // Presence
    8000.0,  // Brilliance
    16000.0, // Air
    20.0,    // Sub-sub-bass (extended)
    45.0,    // Deep bass
    90.0,    // Punch
    180.0,   // Warmth
    350.0,   // Body
    700.0,   // Clarity
];

/// Audio processor with equalizer and enhancement
pub struct AudioProcessor {
    sample_rate: f32,
    eq_enabled: bool,
    eq_gains: [f32; 16],  // In dB
    // Biquad filter coefficients for each band
    filters: Vec<BiquadFilter>,
}

impl AudioProcessor {
    pub fn new(sample_rate: f32) -> Self {
        let mut processor = Self {
            sample_rate,
            eq_enabled: true,
            eq_gains: [0.0; 16],
            filters: Vec::new(),
        };
        processor.update_filters();
        processor
    }

    pub fn set_eq_enabled(&mut self, enabled: bool) {
        self.eq_enabled = enabled;
    }

    pub fn set_eq_gains(&mut self, gains: [f32; 16]) {
        self.eq_gains = gains;
        self.update_filters();
    }

    pub fn set_eq_band(&mut self, band: usize, gain_db: f32) {
        if band < 16 {
            self.eq_gains[band] = gain_db.clamp(-12.0, 12.0);
            self.update_filters();
        }
    }

    fn update_filters(&mut self) {
        self.filters.clear();
        for (i, &freq) in EQUALIZER_FREQUENCIES.iter().enumerate() {
            let gain_db = self.eq_gains[i];
            if gain_db.abs() > 0.1 {  // Only create filter if gain is significant
                self.filters.push(BiquadFilter::peaking_eq(
                    freq,
                    self.sample_rate,
                    0.7,  // Q factor
                    gain_db,
                ));
            }
        }
    }

    /// Process audio samples through the equalizer
    pub fn process(&mut self, samples: &mut [f32]) {
        if !self.eq_enabled || self.filters.is_empty() {
            return;
        }

        for sample in samples.iter_mut() {
            let mut value = *sample;
            for filter in &mut self.filters {
                value = filter.process(value);
            }
            *sample = value;
        }
    }

    /// Process stereo audio (interleaved L/R samples)
    pub fn process_stereo(&mut self, samples: &mut [f32]) {
        if !self.eq_enabled || self.filters.is_empty() {
            return;
        }

        // Process left and right channels
        for i in (0..samples.len()).step_by(2) {
            let mut left = samples[i];
            let mut right = if i + 1 < samples.len() { samples[i + 1] } else { left };

            for filter in &mut self.filters {
                left = filter.process(left);
                right = filter.process(right);
            }

            samples[i] = left;
            if i + 1 < samples.len() {
                samples[i + 1] = right;
            }
        }
    }
}

/// Biquad filter for parametric EQ
struct BiquadFilter {
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    x1: f32, x2: f32,
    y1: f32, y2: f32,
}

impl BiquadFilter {
    /// Create a peaking EQ filter
    fn peaking_eq(freq: f32, sample_rate: f32, q: f32, gain_db: f32) -> Self {
        let a = 10.0_f32.powf(gain_db / 40.0);
        let omega = 2.0 * PI * freq / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_omega;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
        }
    }

    fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1 - self.a2 * self.y2;

        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }
}

/// Spectrum analyzer for visualization
pub struct SpectrumAnalyzer {
    fft_size: usize,
    sample_rate: f32,
    window: Vec<f32>,
    planner: FftPlanner<f32>,
}

impl SpectrumAnalyzer {
    pub fn new(fft_size: usize, sample_rate: f32) -> Self {
        // Hann window for smoother spectrum
        let window: Vec<f32> = (0..fft_size)
            .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / fft_size as f32).cos()))
            .collect();

        Self {
            fft_size,
            sample_rate,
            window,
            planner: FftPlanner::new(),
        }
    }

    /// Analyze spectrum and return magnitude for visualization bars
    /// Returns values normalized to 0.0 - 1.0 range for `num_bars` frequency bands
    pub fn analyze(&mut self, samples: &[f32], num_bars: usize) -> Vec<f32> {
        if samples.len() < self.fft_size {
            return vec![0.0; num_bars];
        }

        // Apply window function
        let mut buffer: Vec<Complex<f32>> = samples
            .iter()
            .take(self.fft_size)
            .zip(self.window.iter())
            .map(|(&s, &w)| Complex::new(s * w, 0.0))
            .collect();

        // Perform FFT
        let fft = self.planner.plan_fft_forward(self.fft_size);
        fft.process(&mut buffer);

        // Calculate magnitude spectrum (only positive frequencies)
        let half_size = self.fft_size / 2;
        let magnitudes: Vec<f32> = buffer[..half_size]
            .iter()
            .map(|c| (c.norm() / half_size as f32).log10().max(-3.0) / 3.0 + 1.0)
            .map(|m| m.clamp(0.0, 1.0))
            .collect();

        // Group into bars (logarithmic frequency scale)
        let mut bars = vec![0.0; num_bars];
        let min_freq = 20.0_f32.log10();
        let max_freq = (self.sample_rate / 2.0).log10();

        for (bar_idx, bar) in bars.iter_mut().enumerate() {
            let low_freq = 10.0_f32.powf(min_freq + (max_freq - min_freq) * bar_idx as f32 / num_bars as f32);
            let high_freq = 10.0_f32.powf(min_freq + (max_freq - min_freq) * (bar_idx + 1) as f32 / num_bars as f32);

            let low_bin = ((low_freq / self.sample_rate) * self.fft_size as f32) as usize;
            let high_bin = ((high_freq / self.sample_rate) * self.fft_size as f32) as usize;

            let low_bin = low_bin.clamp(0, half_size - 1);
            let high_bin = high_bin.clamp(low_bin + 1, half_size);

            // Average magnitude in this frequency range
            let sum: f32 = magnitudes[low_bin..high_bin].iter().sum();
            *bar = sum / (high_bin - low_bin) as f32;
        }

        // Apply some smoothing for visual appeal
        let smoothed: Vec<f32> = bars
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let prev = if i > 0 { bars[i - 1] } else { v };
                let next = if i < bars.len() - 1 { bars[i + 1] } else { v };
                (prev + v * 2.0 + next) / 4.0
            })
            .collect();

        smoothed
    }

    /// Get frequency for a specific bar index
    pub fn get_bar_frequency(&self, bar_idx: usize, num_bars: usize) -> f32 {
        let min_freq = 20.0_f32.log10();
        let max_freq = (self.sample_rate / 2.0).log10();
        10.0_f32.powf(min_freq + (max_freq - min_freq) * (bar_idx as f32 + 0.5) / num_bars as f32)
    }
}

/// Audio enhancement filters
pub struct AudioEnhancer {
    // Loudness normalization
    target_lufs: f32,
    // Dynamic range compression
    compressor_threshold: f32,
    compressor_ratio: f32,
    // High-frequency enhancement
    exciter_amount: f32,
    // Stereo widening
    stereo_width: f32,
}

impl Default for AudioEnhancer {
    fn default() -> Self {
        Self {
            target_lufs: -14.0,
            compressor_threshold: -10.0,
            compressor_ratio: 4.0,
            exciter_amount: 0.1,
            stereo_width: 1.0,
        }
    }
}

impl AudioEnhancer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply soft compression for consistent loudness
    pub fn compress(&self, sample: f32) -> f32 {
        let threshold = 10.0_f32.powf(self.compressor_threshold / 20.0);
        let abs_sample = sample.abs();

        if abs_sample > threshold {
            let excess = abs_sample - threshold;
            let compressed = threshold + excess / self.compressor_ratio;
            compressed.copysign(sample)
        } else {
            sample
        }
    }

    /// Apply harmonic exciter for brightness
    pub fn excite(&self, sample: f32) -> f32 {
        // Simple harmonic generation
        let harmonic = (sample * 2.0).tanh() - sample;
        sample + harmonic * self.exciter_amount
    }

    /// Widen stereo field
    pub fn widen_stereo(&self, left: f32, right: f32) -> (f32, f32) {
        let mid = (left + right) / 2.0;
        let side = (left - right) / 2.0;
        
        let widened_side = side * self.stereo_width;
        
        (mid + widened_side, mid - widened_side)
    }

    /// Process stereo samples with all enhancements
    pub fn process_stereo(&self, left: f32, right: f32) -> (f32, f32) {
        // Compress
        let left = self.compress(left);
        let right = self.compress(right);

        // Excite
        let left = self.excite(left);
        let right = self.excite(right);

        // Widen
        self.widen_stereo(left, right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_processor() {
        let mut processor = AudioProcessor::new(44100.0);
        processor.set_eq_band(0, 6.0);  // Boost bass
        
        let mut samples = vec![0.5, -0.5, 0.3, -0.3];
        processor.process(&mut samples);
        
        // Should have processed without panic
        assert_eq!(samples.len(), 4);
    }

    #[test]
    fn test_spectrum_analyzer() {
        let mut analyzer = SpectrumAnalyzer::new(1024, 44100.0);
        let samples: Vec<f32> = (0..1024).map(|i| (i as f32 * 0.1).sin()).collect();
        
        let bars = analyzer.analyze(&samples, 32);
        assert_eq!(bars.len(), 32);
    }
}
