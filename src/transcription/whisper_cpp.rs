use super::Transcriber;
use crate::{domain::Transcript, Result};
use std::{ffi::OsString, fs, path::{Path, PathBuf}, process::Command};

pub struct WhisperCppCli {
    pub executable: OsString,
    pub model: PathBuf,
}

impl WhisperCppCli {
    fn command(&self, audio: &Path, language: &str, scratch: &Path) -> Command {
        let prefix = scratch.join("transcript");
        let mut cmd = Command::new(&self.executable);
        cmd
            .arg("--model").arg(&self.model)
            .arg("--file").arg(audio)
            .arg("--language").arg(language)
            .arg("--output-txt")
            .arg("--output-file").arg(prefix)
            .arg("--no-timestamps");
        cmd
    }
}

impl Transcriber for WhisperCppCli {
    fn id(&self) -> &'static str { "whisper-cpp-cli" }

    fn preflight(&self) -> Result<()> {
        if !self.model.is_file() {
            return Err(format!("model not found: {}", self.model.display()).into());
        }
        Ok(())
    }

    fn protected_paths(&self) -> Vec<&Path> { vec![self.model.as_path()] }

    fn transcribe(&self, audio: &Path, language: &str, scratch: &Path) -> Result<Transcript> {
        let status = self.command(audio, language, scratch).status()?;
        if !status.success() {
            return Err(format!("whisper.cpp transcription failed ({status})").into());
        }
        let text = fs::read_to_string(scratch.join("transcript.txt"))?;
        // Text-only v0 contract: future structured backends may provide real segments.
        let mut transcript = Transcript::new(
            self.id(),
            (language != "auto").then(|| language.to_owned()),
            text,
        );
        if transcript.text.trim().is_empty() {
            transcript.warnings.push("empty transcript (possibly silence)".to_owned());
        }
        Ok(transcript)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn whisper_arguments_include_model_and_language() {
        let cmd = WhisperCppCli { executable: "whisper-cli".into(), model: "model.bin".into() }
            .command(Path::new("audio.wav"), "fr", Path::new("/tmp"));
        let args: Vec<_> = cmd.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        assert!(args.windows(2).any(|a| a[0] == "--model" && a[1] == "model.bin"));
        assert!(args.windows(2).any(|a| a[0] == "--language" && a[1] == "fr"));
        assert!(args.contains(&"--output-txt".into()));
    }
}
