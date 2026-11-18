//! Audio decoding and normalization: always produce a WAV accepted by the selected STT.
mod ffmpeg;
mod wav_pcm16;
use crate::Result;
pub use ffmpeg::Ffmpeg;
use std::path::Path;
pub use wav_pcm16::WavPcm16;

pub trait AudioDecoder: Send + Sync {
    fn normalize(&self, source: &Path, destination: &Path) -> Result<()>;
}
