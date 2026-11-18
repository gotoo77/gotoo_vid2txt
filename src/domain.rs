use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Explicit input source. URL handling never happens implicitly inside FFmpeg.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Local(PathBuf),
    Url(String),
}

impl Source {
    pub fn parse(value: &str) -> Self {
        if value.starts_with("https://") || value.starts_with("http://") {
            Self::Url(value.to_owned())
        } else {
            Self::Local(PathBuf::from(value))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Job {
    pub source: Source,
    pub language: String,
    pub output: PathBuf,
    pub audio_output: Option<PathBuf>,
    pub force: bool,
}

/// Versioned, engine-independent result format. No URL or model path is emitted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transcript {
    pub schema_version: u32,
    pub backend: String,
    pub language: Option<String>,
    pub text: String,
    pub segments: Vec<Segment>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Segment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

impl Transcript {
    pub fn new(backend: impl Into<String>, language: Option<String>, text: String) -> Self {
        Self {
            schema_version: 1,
            backend: backend.into(),
            language,
            text,
            segments: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_http_schemes_are_urls() {
        assert!(matches!(Source::parse("https://example.org/a"), Source::Url(_)));
        assert!(matches!(Source::parse("http://example.org/a"), Source::Url(_)));
        assert!(matches!(Source::parse("file:///tmp/a.mp4"), Source::Local(_)));
        assert!(matches!(Source::parse("video.mp4"), Source::Local(_)));
    }

    #[test]
    fn transcript_contract_has_schema_version() {
        let value = Transcript::new("fake", Some("fr".into()), "bonjour".into());
        let json = serde_json::to_value(value).unwrap();
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["text"], "bonjour");
        assert_eq!(json["segments"].as_array().unwrap().len(), 0);
    }
}
