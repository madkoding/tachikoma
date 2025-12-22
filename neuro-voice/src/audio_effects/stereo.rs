//! =============================================================================
//! Stereo Effects Module
//! =============================================================================
//! Creates stereo field from mono audio with dual-voice panning effect.
//! Simulates two voices alternating between L/R channels.
//! =============================================================================

use super::SAMPLE_RATE;

/// Stereo audio buffer (Left, Right channels)
pub struct StereoBuffer {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
}

impl StereoBuffer {
    pub fn new(len: usize) -> Self {
        Self {
            left: vec![0.0; len],
            right: vec![0.0; len],
        }
    }

    /// Interleave L/R channels for standard stereo output
    pub fn interleave(&self) -> Vec<f32> {
        let mut output = Vec::with_capacity(self.left.len() * 2);
        for i in 0..self.left.len() {
            output.push(self.left[i]);
            output.push(self.right[i]);
        }
        output
    }
}

/// Create stereo from mono with dual-voice effect
/// 
/// Creates the illusion of two voices by:
/// - Left channel: Original processed audio
/// - Right channel: Slightly delayed + pitch-varied version
/// 
/// The voices "trade" positions periodically for dynamic stereo field
pub fn mono_to_stereo_dual_voice(
    mono: &[f32],
    delay_ms: f32,      // Delay for right channel (5-15ms typical)
    pitch_detune: f32,  // Pitch difference in cents (10-30 typical)
    swap_rate_hz: f32,  // How often voices swap channels (0.5-2 Hz)
) -> StereoBuffer {
    let delay_samples = ((delay_ms / 1000.0) * SAMPLE_RATE as f32) as usize;
    let len = mono.len();
    
    let mut stereo = StereoBuffer::new(len);
    
    // Create detuned version for "second voice"
    let detuned = apply_micro_pitch_shift(mono, pitch_detune);
    
    // Calculate swap envelope (smooth crossfade between channels)
    let swap_period = (SAMPLE_RATE as f32 / swap_rate_hz) as usize;
    
    for i in 0..len {
        // Original sample
        let original = mono[i];
        
        // Delayed + detuned sample (with bounds check)
        let delayed_idx = if i >= delay_samples { i - delay_samples } else { 0 };
        let detuned_sample = detuned[delayed_idx];
        
        // Calculate crossfade position (0.0 to 1.0, oscillating)
        let phase = (i % swap_period) as f32 / swap_period as f32;
        let crossfade = (phase * std::f32::consts::PI * 2.0).sin() * 0.5 + 0.5;
        
        // Smooth crossfade between channel assignments
        // When crossfade = 0: original→L, detuned→R
        // When crossfade = 1: detuned→L, original→R
        stereo.left[i] = original * (1.0 - crossfade) + detuned_sample * crossfade;
        stereo.right[i] = detuned_sample * (1.0 - crossfade) + original * crossfade;
    }
    
    stereo
}

/// Create stereo with Haas effect (precedence effect)
/// More subtle than dual-voice, creates width without obvious movement
pub fn mono_to_stereo_haas(mono: &[f32], delay_ms: f32) -> StereoBuffer {
    let delay_samples = ((delay_ms / 1000.0) * SAMPLE_RATE as f32) as usize;
    let len = mono.len();
    
    let mut stereo = StereoBuffer::new(len);
    
    for i in 0..len {
        // Left: original
        stereo.left[i] = mono[i];
        
        // Right: delayed (creates width perception)
        let delayed_idx = if i >= delay_samples { i - delay_samples } else { 0 };
        stereo.right[i] = mono[delayed_idx];
    }
    
    stereo
}

/// Create stereo with dynamic panning (voice moves L→R→L)
pub fn mono_to_stereo_autopan(
    mono: &[f32],
    pan_rate_hz: f32,  // How fast the voice moves (0.2-1.0 Hz typical)
    pan_depth: f32,    // How wide the movement (0.0-1.0, where 1.0 = full L↔R)
) -> StereoBuffer {
    let len = mono.len();
    let mut stereo = StereoBuffer::new(len);
    
    for i in 0..len {
        // Calculate pan position (-1 to +1)
        let t = i as f32 / SAMPLE_RATE as f32;
        let pan_position = (t * pan_rate_hz * std::f32::consts::PI * 2.0).sin() * pan_depth;
        
        // Convert pan position to L/R gains (constant power panning)
        let angle = (pan_position + 1.0) * std::f32::consts::FRAC_PI_4; // 0 to π/2
        let left_gain = angle.cos();
        let right_gain = angle.sin();
        
        stereo.left[i] = mono[i] * left_gain;
        stereo.right[i] = mono[i] * right_gain;
    }
    
    stereo
}

/// Simple micro pitch shift using resampling (for small detuning)
/// cents: pitch change in cents (100 cents = 1 semitone)
fn apply_micro_pitch_shift(audio: &[f32], cents: f32) -> Vec<f32> {
    if cents.abs() < 0.1 {
        return audio.to_vec();
    }
    
    // Convert cents to ratio: 2^(cents/1200)
    let ratio = 2.0_f32.powf(cents / 1200.0);
    let new_len = (audio.len() as f32 / ratio) as usize;
    let mut output = Vec::with_capacity(new_len);
    
    for i in 0..new_len {
        let src_pos = i as f32 * ratio;
        let src_idx = src_pos as usize;
        let frac = src_pos - src_idx as f32;
        
        let sample = if src_idx + 1 < audio.len() {
            audio[src_idx] * (1.0 - frac) + audio[src_idx + 1] * frac
        } else {
            audio[src_idx.min(audio.len() - 1)]
        };
        
        output.push(sample);
    }
    
    // Pad or trim to match original length
    output.resize(audio.len(), 0.0);
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mono_to_stereo_dual_voice() {
        let mono = vec![0.5; 1000];
        let stereo = mono_to_stereo_dual_voice(&mono, 10.0, 20.0, 1.0);
        
        assert_eq!(stereo.left.len(), mono.len());
        assert_eq!(stereo.right.len(), mono.len());
    }

    #[test]
    fn test_interleave() {
        let mut stereo = StereoBuffer::new(2);
        stereo.left = vec![1.0, 3.0];
        stereo.right = vec![2.0, 4.0];
        
        let interleaved = stereo.interleave();
        assert_eq!(interleaved, vec![1.0, 2.0, 3.0, 4.0]);
    }
}
