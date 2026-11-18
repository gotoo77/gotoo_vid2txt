//! Audio decoding and normalization: always produce a WAV accepted by the selected STT.
mod ffmpeg;
mod wav_pcm16;
pub use ffmpeg::Ffmpeg;
pub use wav_pcm16::WavPcm16;
use crate::Result;
use std::path::Path;

pub trait AudioDecoder: Send + Sync {
    fn normalize(&self, source: &Path, destination: &Path) -> Result<()>;
}
