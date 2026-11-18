//! Zero-transcode adapter for *already normalized* RIFF/WAVE PCM16 mono 16kHz.
//! This adapter intentionally rejects everything else rather than guessing.
use super::AudioDecoder;
use crate::Result;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

pub struct WavPcm16;

fn validate_header(input: &Path) -> Result<()> {
    let mut file = File::open(input)?;
    let mut header = [0_u8; 44];
    file.read_exact(&mut header)?;
    let u16le = |offset: usize| u16::from_le_bytes([header[offset], header[offset + 1]]);
    let u32le = |offset: usize| u32::from_le_bytes(header[offset..offset + 4].try_into().unwrap());
    // Deliberately accept just a canonical 44-byte WAV header (fmt 16 then data).
    // Complex but valid WAV layouts must use a full decoder (e.g. FFmpeg).
    let valid = &header[0..4] == b"RIFF"
        && &header[8..12] == b"WAVE"
        && &header[12..16] == b"fmt "
        && u32le(16) == 16
        && u16le(20) == 1
        && u16le(22) == 1
        && u32le(24) == 16_000
        && u32le(28) == 32_000
        && u16le(32) == 2
        && u16le(34) == 16
        && &header[36..40] == b"data";
    if !valid {
        return Err("wav-pcm16 requires canonical 16 kHz mono PCM16 WAV".into());
    }
    let data_len = u64::from(u32le(40));
    if data_len == 0
        || data_len % 2 != 0
        || u64::from(u32le(4)) != 36 + data_len
        || file.metadata()?.len() != 44 + data_len
    {
        return Err("invalid WAV data size".into());
    }
    Ok(())
}

impl AudioDecoder for WavPcm16 {
    fn normalize(&self, source: &Path, destination: &Path) -> Result<()> {
        validate_header(source)?;
        let mut input = File::open(source)?;
        let mut target = File::create_new(destination)?;
        std::io::copy(&mut input, &mut target)?;
        target.flush()?;
        // We deliberately preserve the verified input WAV bytes.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    fn wav() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(b"RIFF");
        b.extend_from_slice(&38u32.to_le_bytes());
        b.extend_from_slice(b"WAVEfmt ");
        b.extend_from_slice(&16u32.to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&1u16.to_le_bytes());
        b.extend_from_slice(&16_000u32.to_le_bytes());
        b.extend_from_slice(&32_000u32.to_le_bytes());
        b.extend_from_slice(&2u16.to_le_bytes());
        b.extend_from_slice(&16u16.to_le_bytes());
        b.extend_from_slice(b"data");
        b.extend_from_slice(&2u32.to_le_bytes());
        b.extend_from_slice(&[0, 0]);
        b
    }
    #[test]
    fn valid_wav_is_copied_exactly() {
        let t = tempfile::tempdir().unwrap();
        let source = t.path().join("in.wav");
        let output = t.path().join("out.wav");
        fs::write(&source, wav()).unwrap();
        WavPcm16.normalize(&source, &output).unwrap();
        assert_eq!(fs::read(&source).unwrap(), fs::read(output).unwrap());
    }
    #[test]
    fn wrong_rate_or_truncated_payload_is_rejected() {
        let t = tempfile::tempdir().unwrap();
        let source = t.path().join("in.wav");
        let output = t.path().join("out.wav");
        let mut data = wav();
        data[24..28].copy_from_slice(&48_000u32.to_le_bytes());
        fs::write(&source, &data).unwrap();
        assert!(WavPcm16.normalize(&source, &output).is_err());
        fs::write(&source, &wav()[..44]).unwrap();
        assert!(WavPcm16.normalize(&source, &output).is_err());
        assert!(!output.exists());
    }
}
