//! =============================================================================
//! Opus Encoder Module
//! =============================================================================
//! Encodes PCM audio to Opus format in OGG container for browser compatibility.
//! Opus provides ~10x compression vs WAV with minimal latency (~5ms).
//! Supports stereo dual-voice effect for spatial audio experience.
//! =============================================================================

use anyhow::{anyhow, Result};
use ogg::writing::PacketWriter;
use opus::{Application, Channels, Encoder};
use std::borrow::Cow;
use std::io::Cursor;
use tracing::{debug, error};

/// Sample rate for Opus encoding (must be one of: 8000, 12000, 16000, 24000, 48000)
/// Using 48kHz for highest quality (native Opus rate)
pub const OPUS_SAMPLE_RATE: u32 = 48000;

/// Frame size in samples per channel (20ms at 48kHz = 960 samples per channel)
/// Opus supports 2.5, 5, 10, 20, 40, 60 ms frames
/// 20ms is optimal balance between latency and compression
pub const OPUS_FRAME_SIZE: usize = 960;

/// Stereo frame size (960 samples * 2 channels = 1920 interleaved samples)
pub const OPUS_STEREO_FRAME_SIZE: usize = OPUS_FRAME_SIZE * 2;

/// Bitrate for high quality stereo voice (64-96 kbps for excellent stereo speech)
pub const OPUS_BITRATE: i32 = 64000;

/// Create OpusHead header packet (required for OGG Opus)
/// Configured for stereo (2 channels)
fn create_opus_head() -> Vec<u8> {
    let mut head = Vec::with_capacity(19);
    head.extend_from_slice(b"OpusHead");      // Magic signature
    head.push(1);                              // Version
    head.push(2);                              // Channel count (stereo)
    head.extend_from_slice(&0u16.to_le_bytes()); // Pre-skip (samples)
    head.extend_from_slice(&OPUS_SAMPLE_RATE.to_le_bytes()); // Input sample rate
    head.extend_from_slice(&0i16.to_le_bytes()); // Output gain
    head.push(0);                              // Channel mapping family (0 = mono/stereo)
    head
}

/// Create OpusTags comment header packet
fn create_opus_tags() -> Vec<u8> {
    let vendor = "NEURO-voice";
    let mut tags = Vec::with_capacity(16 + vendor.len());
    tags.extend_from_slice(b"OpusTags");
    tags.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
    tags.extend_from_slice(vendor.as_bytes());
    tags.extend_from_slice(&0u32.to_le_bytes()); // User comment list length (0)
    tags
}

/// Encode stereo PCM samples to OGG/Opus format (browser-compatible)
/// 
/// # Arguments
/// * `pcm_samples` - Input PCM samples (stereo interleaved L,R,L,R, f32, 48kHz expected)
/// 
/// # Returns
/// * `Vec<u8>` - OGG/Opus encoded stereo audio that browsers can decode natively
pub fn encode_pcm_to_opus(pcm_samples: &[f32]) -> Result<Vec<u8>> {
    if pcm_samples.is_empty() {
        return Ok(Vec::new());
    }

    // Create stereo encoder for voice application
    let mut encoder = Encoder::new(OPUS_SAMPLE_RATE, Channels::Stereo, Application::Voip)
        .map_err(|e| anyhow!("Failed to create Opus encoder: {}", e))?;

    // Set bitrate (higher for stereo)
    encoder.set_bitrate(opus::Bitrate::Bits(OPUS_BITRATE))
        .map_err(|e| anyhow!("Failed to set bitrate: {}", e))?;

    // Enable variable bitrate for better quality
    encoder.set_vbr(true)
        .map_err(|e| anyhow!("Failed to enable VBR: {}", e))?;

    // Output buffer for OGG
    let mut output = Vec::new();
    let cursor = Cursor::new(&mut output);
    let mut writer = PacketWriter::new(cursor);

    // Serial number for OGG stream
    let serial = 1u32;

    // Write OpusHead header (BOS - beginning of stream)
    let opus_head = create_opus_head();
    writer.write_packet(Cow::Owned(opus_head), serial, ogg::writing::PacketWriteEndInfo::EndPage, 0)
        .map_err(|e| anyhow!("Failed to write OpusHead: {}", e))?;

    // Write OpusTags header
    let opus_tags = create_opus_tags();
    writer.write_packet(Cow::Owned(opus_tags), serial, ogg::writing::PacketWriteEndInfo::EndPage, 0)
        .map_err(|e| anyhow!("Failed to write OpusTags: {}", e))?;

    // Buffer for encoded frame (max ~4KB per frame, but typically much smaller)
    let mut frame_buffer = vec![0u8; 4000];

    // Granule position (cumulative samples per channel)
    let mut granule_pos: u64 = 0;

    // For stereo: each frame needs OPUS_FRAME_SIZE samples per channel
    // Interleaved format: [L0, R0, L1, R1, ...] so we need OPUS_FRAME_SIZE * 2 values per frame
    let stereo_frame_size = OPUS_STEREO_FRAME_SIZE;
    let chunks: Vec<_> = pcm_samples.chunks(stereo_frame_size).collect();
    let total_chunks = chunks.len();

    for (idx, chunk) in chunks.into_iter().enumerate() {
        // Pad last chunk if necessary (must be exact stereo frame size)
        let input: Vec<f32> = if chunk.len() < stereo_frame_size {
            let mut padded = chunk.to_vec();
            padded.resize(stereo_frame_size, 0.0);
            padded
        } else {
            chunk.to_vec()
        };

        // Encode stereo frame (encoder expects OPUS_FRAME_SIZE samples, 
        // but input has 2x for stereo interleaved)
        match encoder.encode_float(&input, &mut frame_buffer) {
            Ok(len) => {
                // Granule position tracks samples per channel (not total samples)
                granule_pos += OPUS_FRAME_SIZE as u64;
                
                // Determine if this is the last packet
                let end_info = if idx == total_chunks - 1 {
                    ogg::writing::PacketWriteEndInfo::EndStream
                } else {
                    ogg::writing::PacketWriteEndInfo::NormalPacket
                };

                writer.write_packet(
                    Cow::Owned(frame_buffer[..len].to_vec()),
                    serial,
                    end_info,
                    granule_pos,
                ).map_err(|e| anyhow!("Failed to write Opus frame: {}", e))?;
            }
            Err(e) => {
                error!("Opus encoding error: {}", e);
                return Err(anyhow!("Opus encoding failed: {}", e));
            }
        }
    }

    // Get the final output
    drop(writer);
    
    debug!(
        "Encoded {} stereo samples to {} bytes OGG/Opus (compression: {:.1}x)",
        pcm_samples.len() / 2, // Samples per channel
        output.len(),
        (pcm_samples.len() * 4) as f32 / output.len().max(1) as f32
    );

    Ok(output)
}

/// Resample mono audio from source rate to target rate (simple linear interpolation)
/// For higher quality, consider using a proper resampler like rubato
pub fn resample(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    if source_rate == target_rate {
        return samples.to_vec();
    }

    let ratio = source_rate as f64 / target_rate as f64;
    let new_len = (samples.len() as f64 / ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_pos = i as f64 * ratio;
        let src_idx = src_pos as usize;
        let frac = src_pos - src_idx as f64;

        let sample = if src_idx + 1 < samples.len() {
            samples[src_idx] * (1.0 - frac as f32) + samples[src_idx + 1] * frac as f32
        } else {
            samples[src_idx.min(samples.len() - 1)]
        };

        resampled.push(sample);
    }

    resampled
}

/// Resample stereo interleaved audio (L,R,L,R) from source rate to target rate
pub fn resample_stereo(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    if source_rate == target_rate {
        return samples.to_vec();
    }

    // Separate channels
    let num_frames = samples.len() / 2;
    let mut left = Vec::with_capacity(num_frames);
    let mut right = Vec::with_capacity(num_frames);
    
    for frame in samples.chunks_exact(2) {
        left.push(frame[0]);
        right.push(frame[1]);
    }

    // Resample each channel independently
    let left_resampled = resample(&left, source_rate, target_rate);
    let right_resampled = resample(&right, source_rate, target_rate);

    // Interleave back
    let mut result = Vec::with_capacity(left_resampled.len() * 2);
    for (l, r) in left_resampled.iter().zip(right_resampled.iter()) {
        result.push(*l);
        result.push(*r);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_stereo_silence() {
        // 1 second of stereo silence at 48kHz (48000 samples * 2 channels)
        let silence = vec![0.0f32; 96000];
        let encoded = encode_pcm_to_opus(&silence).unwrap();
        
        // Should be much smaller than raw PCM (96000 * 4 = 384KB)
        assert!(encoded.len() < 20000, "Encoded size should be < 20KB, got {}", encoded.len());
        
        // Should start with OGG magic header
        assert_eq!(&encoded[0..4], b"OggS");
    }

    #[test]
    fn test_resample_mono() {
        let input = vec![0.0, 1.0, 0.0, -1.0]; // Simple wave at source rate
        let resampled = resample(&input, 22050, 24000);
        
        // Should have approximately (24000/22050) * 4 samples
        let expected_len = (4.0 * 24000.0 / 22050.0) as usize;
        assert!((resampled.len() as i32 - expected_len as i32).abs() <= 1);
    }

    #[test]
    fn test_resample_stereo() {
        // Simple stereo wave: L=[0, 1, 0, -1], R=[1, 0, -1, 0]
        let input = vec![0.0, 1.0, 1.0, 0.0, 0.0, -1.0, -1.0, 0.0];
        let resampled = resample_stereo(&input, 22050, 24000);
        
        // Should have approximately (24000/22050) * 4 frames * 2 channels
        let expected_frames = (4.0 * 24000.0 / 22050.0) as usize;
        let expected_samples = expected_frames * 2;
        assert!((resampled.len() as i32 - expected_samples as i32).abs() <= 2);
        
        // Should still be even (stereo pairs)
        assert_eq!(resampled.len() % 2, 0);
    }
}
