//! Compile-time backend registry. No dynamic library loading and no hidden fallbacks.
//! Adding an implementation requires an explicit match arm and contract tests.
use crate::{
    acquisition::{Acquisition, LocalAcquisition, YtDlp},
    audio::{AudioDecoder, Ffmpeg, WavPcm16},
    config::{AudioBackend, Config, ExportBackend, SourceBackend, TranscriptionBackend},
    domain::{Source, Transcript},
    export::{Exporter, Json, PlainText},
    pipeline::Pipeline,
    transcription::{Transcriber, WhisperCppCli},
    Result,
};
use std::path::{Path, PathBuf};

pub enum SelectedAcquisition {
    Auto { remote: YtDlp },
    Local(LocalAcquisition),
    Remote(YtDlp),
}
impl Acquisition for SelectedAcquisition {
    fn acquire(&self, source: &Source, scratch: &Path) -> Result<PathBuf> {
        match (self, source) {
            (Self::Auto { .. }, Source::Local(_)) | (Self::Local(_), Source::Local(_)) => {
                LocalAcquisition.acquire(source, scratch)
            }
            (Self::Auto { remote }, Source::Url(_)) | (Self::Remote(remote), Source::Url(_)) => {
                remote.acquire(source, scratch)
            }
            (Self::Local(_), Source::Url(_)) => {
                Err("local acquisition backend cannot accept URLs".into())
            }
            (Self::Remote(_), Source::Local(_)) => {
                Err("yt-dlp acquisition backend requires an URL".into())
            }
        }
    }
}

pub enum SelectedAudio {
    Ffmpeg(Ffmpeg),
    WavPcm16(WavPcm16),
}
impl AudioDecoder for SelectedAudio {
    fn normalize(&self, input: &Path, output: &Path) -> Result<()> {
        match self {
            Self::Ffmpeg(v) => v.normalize(input, output),
            Self::WavPcm16(v) => v.normalize(input, output),
        }
    }
}

pub enum SelectedTranscriber {
    WhisperCpp(WhisperCppCli),
}
impl Transcriber for SelectedTranscriber {
    fn id(&self) -> &'static str {
        match self {
            Self::WhisperCpp(v) => v.id(),
        }
    }
    fn preflight(&self) -> Result<()> {
        match self {
            Self::WhisperCpp(v) => v.preflight(),
        }
    }
    fn protected_paths(&self) -> Vec<&Path> {
        match self {
            Self::WhisperCpp(v) => v.protected_paths(),
        }
    }
    fn transcribe(&self, audio: &Path, language: &str, scratch: &Path) -> Result<Transcript> {
        match self {
            Self::WhisperCpp(v) => v.transcribe(audio, language, scratch),
        }
    }
}

pub enum SelectedExporter {
    Text(PlainText),
    Json(Json),
}
impl Exporter for SelectedExporter {
    fn id(&self) -> &'static str {
        match self {
            Self::Text(v) => v.id(),
            Self::Json(v) => v.id(),
        }
    }
    fn extension(&self) -> &'static str {
        match self {
            Self::Text(v) => v.extension(),
            Self::Json(v) => v.extension(),
        }
    }
    fn render(&self, t: &Transcript) -> Result<Vec<u8>> {
        match self {
            Self::Text(v) => v.render(t),
            Self::Json(v) => v.render(t),
        }
    }
}

pub type RuntimePipeline =
    Pipeline<SelectedAcquisition, SelectedAudio, SelectedTranscriber, SelectedExporter>;

pub fn validate_source(mode: SourceBackend, source: &Source) -> Result<&'static str> {
    match (mode, source) {
        (SourceBackend::Auto | SourceBackend::Local, Source::Local(_)) => Ok("local"),
        (SourceBackend::Auto | SourceBackend::YtDlp, Source::Url(_)) => Ok("yt-dlp"),
        _ => Err("configured acquisition backend cannot handle this source type".into()),
    }
}

pub fn build(config: &Config) -> Result<RuntimePipeline> {
    let profile = &config.profile.default;
    let remote = || YtDlp {
        executable: config.backends.yt_dlp.binary.clone().into_os_string(),
    };
    let acquisition = match profile.source {
        SourceBackend::Auto => SelectedAcquisition::Auto { remote: remote() },
        SourceBackend::Local => SelectedAcquisition::Local(LocalAcquisition),
        SourceBackend::YtDlp => SelectedAcquisition::Remote(remote()),
    };
    let decoder = match profile.audio {
        AudioBackend::Ffmpeg => SelectedAudio::Ffmpeg(Ffmpeg {
            executable: config.backends.ffmpeg.binary.clone().into_os_string(),
        }),
        AudioBackend::WavPcm16 => SelectedAudio::WavPcm16(WavPcm16),
    };
    let transcriber = match profile.transcriber {
        TranscriptionBackend::WhisperCpp => SelectedTranscriber::WhisperCpp(WhisperCppCli {
            executable: config
                .backends
                .whisper_cpp
                .binary
                .clone()
                .unwrap_or_else(|| PathBuf::from("whisper-cli"))
                .into_os_string(),
            model: config.backends.whisper_cpp.model.clone().ok_or(
                "whisper_cpp requires a model: set [backends.whisper_cpp].model or --model",
            )?,
        }),
    };
    let exporter = match profile.exporter {
        ExportBackend::Txt => SelectedExporter::Text(PlainText),
        ExportBackend::Json => SelectedExporter::Json(Json),
    };
    Ok(Pipeline {
        acquisition,
        decoder,
        transcriber,
        exporter,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_source_mismatches_fail_early() {
        assert!(validate_source(
            SourceBackend::Local,
            &Source::Url("https://example.org".into())
        )
        .is_err());
        assert!(validate_source(SourceBackend::YtDlp, &Source::Local("file.wav".into())).is_err());
        assert_eq!(
            validate_source(SourceBackend::Auto, &Source::Local("file.wav".into())).unwrap(),
            "local"
        );
    }
    #[test]
    fn registry_selects_audio_and_exporter_without_changing_pipeline() {
        let mut c = Config::default();
        c.profile.default.audio = AudioBackend::WavPcm16;
        c.profile.default.exporter = ExportBackend::Json;
        c.backends.whisper_cpp.model = Some("model.bin".into());
        let p = build(&c).unwrap();
        assert!(matches!(p.decoder, SelectedAudio::WavPcm16(_)));
        assert_eq!(p.exporter.id(), "json");
    }
    #[test]
    fn missing_model_cannot_be_silently_ignored() {
        assert!(build(&Config::default()).is_err());
    }
}
