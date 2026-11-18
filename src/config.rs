//! Versioned-ish human configuration. Invalid or unknown settings are rejected.
//! Only declared, implemented backends may be selected.
use crate::Result;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
#[clap(rename_all = "kebab-case")]
pub enum SourceBackend {
    Auto,
    Local,
    YtDlp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
#[clap(rename_all = "kebab-case")]
pub enum AudioBackend {
    Ffmpeg,
    WavPcm16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
#[clap(rename_all = "kebab-case")]
pub enum TranscriptionBackend {
    WhisperCpp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
#[clap(rename_all = "kebab-case")]
pub enum ExportBackend {
    Txt,
    Json,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Profile {
    pub source: SourceBackend,
    pub audio: AudioBackend,
    pub transcriber: TranscriptionBackend,
    pub exporter: ExportBackend,
}
impl Default for Profile {
    fn default() -> Self {
        Self {
            source: SourceBackend::Auto,
            audio: AudioBackend::Ffmpeg,
            transcriber: TranscriptionBackend::WhisperCpp,
            exporter: ExportBackend::Txt,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Profiles {
    pub default: Profile,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Executable {
    pub binary: PathBuf,
}
impl Executable {
    fn named(binary: &str) -> Self {
        Self {
            binary: PathBuf::from(binary),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct WhisperConfig {
    pub binary: Option<PathBuf>,
    pub model: Option<PathBuf>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Backends {
    pub yt_dlp: Executable,
    pub ffmpeg: Executable,
    pub whisper_cpp: WhisperConfig,
}
impl Default for Backends {
    fn default() -> Self {
        Self {
            yt_dlp: Executable::named("yt-dlp"),
            ffmpeg: Executable::named("ffmpeg"),
            whisper_cpp: WhisperConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub profile: Profiles,
    pub backends: Backends,
}

/// CLI flags override the specified config values only (not the whole profile).
#[derive(Debug, Clone, Default)]
pub struct Overrides {
    pub source: Option<SourceBackend>,
    pub audio: Option<AudioBackend>,
    pub transcriber: Option<TranscriptionBackend>,
    pub exporter: Option<ExportBackend>,
    pub yt_dlp_bin: Option<PathBuf>,
    pub ffmpeg_bin: Option<PathBuf>,
    pub whisper_bin: Option<PathBuf>,
    pub model: Option<PathBuf>,
}

impl Config {
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let Some(path) = path else {
            return Ok(Self::default());
        };
        let contents = fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }
    pub fn apply(&mut self, over: Overrides) {
        let p = &mut self.profile.default;
        if let Some(v) = over.source {
            p.source = v;
        }
        if let Some(v) = over.audio {
            p.audio = v;
        }
        if let Some(v) = over.transcriber {
            p.transcriber = v;
        }
        if let Some(v) = over.exporter {
            p.exporter = v;
        }
        if let Some(v) = over.yt_dlp_bin {
            self.backends.yt_dlp.binary = v;
        }
        if let Some(v) = over.ffmpeg_bin {
            self.backends.ffmpeg.binary = v;
        }
        if let Some(v) = over.whisper_bin {
            self.backends.whisper_cpp.binary = Some(v);
        }
        if let Some(v) = over.model {
            self.backends.whisper_cpp.model = Some(v);
        }
    }
}

impl ExportBackend {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Txt => "txt",
            Self::Json => "json",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_config_uses_explicit_defaults() {
        let c: Config = toml::from_str("").unwrap();
        assert_eq!(c.profile.default.source, SourceBackend::Auto);
        assert_eq!(c.profile.default.audio, AudioBackend::Ffmpeg);
        assert_eq!(c.profile.default.exporter, ExportBackend::Txt);
    }
    #[test]
    fn parser_accepts_nested_backend_settings() {
        let config: Config = toml::from_str(
            r#"
            [profile.default]
            source = "local"
            audio = "wav_pcm16"
            exporter = "json"
            [backends.whisper_cpp]
            model = "models/base.bin"
        "#,
        )
        .unwrap();
        assert_eq!(config.profile.default.audio, AudioBackend::WavPcm16);
        assert_eq!(
            config.backends.whisper_cpp.model.unwrap(),
            PathBuf::from("models/base.bin")
        );
    }
    #[test]
    fn invalid_backend_and_typos_fail_closed() {
        assert!(toml::from_str::<Config>("[profile.default]\naudio = 'unknown'").is_err());
        assert!(toml::from_str::<Config>("[profile.default]\naudo = 'ffmpeg'").is_err());
        assert!(toml::from_str::<Config>("[backends.whisper_cpp]\ntoken = 'secret'").is_err());
    }
    #[test]
    fn cli_overrides_selected_values_only() {
        let mut c: Config =
            toml::from_str("[profile.default]\nsource='local'\nexporter='json'").unwrap();
        c.apply(Overrides {
            audio: Some(AudioBackend::WavPcm16),
            ..Overrides::default()
        });
        assert_eq!(c.profile.default.source, SourceBackend::Local);
        assert_eq!(c.profile.default.audio, AudioBackend::WavPcm16);
        assert_eq!(c.profile.default.exporter, ExportBackend::Json);
    }
}
