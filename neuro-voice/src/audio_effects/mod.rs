//! =============================================================================
//! Audio Effects Module - DSP Effects for voice transformation
//! =============================================================================

/// Output sample rate for high quality audio (44.1kHz CD quality)
pub const SAMPLE_RATE: u32 = 44100;

/// Piper TTS native sample rate (fixed by model)
#[allow(dead_code)]
pub const PIPER_SAMPLE_RATE: u32 = 22050;

mod pitch_shift;
mod chorus;
mod reverb;
mod limiter;
mod chain;
mod stereo;

pub use pitch_shift::apply_pitch_shift;
pub use chain::apply_robot_effect_chain;
pub use stereo::{mono_to_stereo_dual_voice, StereoBuffer};
