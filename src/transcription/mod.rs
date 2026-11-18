//! Backend contract independent from whisper.cpp, ONNX, cloud APIs and MCP.
mod whisper_cpp;
use crate::{domain::Transcript, Result};
use std::path::Path;
pub use whisper_cpp::WhisperCppCli;

pub trait Transcriber: Send + Sync {
    fn id(&self) -> &'static str;
    /// Validate engine-specific requirements before acquisition or decoding.
    fn preflight(&self) -> Result<()> {
        Ok(())
    }
    /// Paths used by this adapter that must never become output paths.
    fn protected_paths(&self) -> Vec<&Path> {
        Vec::new()
    }
    fn transcribe(&self, audio: &Path, language: &str, scratch: &Path) -> Result<Transcript>;
}
