//! Extensible video-to-transcript pipeline.
//!
//! - Acquisition, decoding, transcription and publication are independently testable.
//! - The public domain model deliberately does not depend on a speech-engine SDK.
//! - This release is an **unvalidated alpha candidate**, not yet a stable API.

pub mod acquisition;
pub mod audio;
pub mod config;
pub mod export;
pub mod registry;
pub mod domain;
pub mod output;
pub mod pipeline;
pub mod transcription;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
