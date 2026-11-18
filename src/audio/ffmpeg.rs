use super::AudioDecoder;
use crate::Result;
use std::{ffi::OsString, path::Path, process::Command};

pub struct Ffmpeg {
    pub executable: OsString,
}

impl Ffmpeg {
    fn command(&self, source: &Path, destination: &Path) -> Command {
        let mut cmd = Command::new(&self.executable);
        cmd
            .arg("-nostdin").arg("-hide_banner")
            .arg("-loglevel").arg("error")
            .arg("-y")
            .arg("-i").arg(source)
            .arg("-vn")
            .arg("-ar").arg("16000")
            .arg("-ac").arg("1")
            .arg("-c:a").arg("pcm_s16le")
            .arg("-f").arg("wav")
            .arg(destination);
        cmd
    }
}

impl AudioDecoder for Ffmpeg {
    fn normalize(&self, source: &Path, destination: &Path) -> Result<()> {
        let status = self.command(source, destination).status()?;
        if !status.success() {
            return Err(format!("ffmpeg normalization failed ({status})").into());
        }
        if !destination.is_file() {
            return Err("ffmpeg did not produce an audio file".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ffmpeg_uses_explicit_pcm_wav_contract() {
        let args: Vec<_> = Ffmpeg { executable: "ffmpeg".into() }
            .command(Path::new("input.webm"), Path::new("out.wav"))
            .get_args().map(|a| a.to_string_lossy().to_string()).collect();
        assert!(args.windows(2).any(|a| a[0] == "-ar" && a[1] == "16000"));
        assert!(args.windows(2).any(|a| a[0] == "-ac" && a[1] == "1"));
        assert!(args.windows(2).any(|a| a[0] == "-c:a" && a[1] == "pcm_s16le"));
        assert!(args.contains(&"-nostdin".into()));
    }
}
