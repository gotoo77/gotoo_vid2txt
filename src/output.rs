//! Output publication. A file is staged beside its target before publication.
//! Two requested outputs are NOT a single atomic transaction.
use crate::Result;
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

fn normalized_target(path: &Path) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let name = absolute.file_name().ok_or("output must have a file name")?;
    let parent = absolute
        .parent()
        .ok_or("output must have a parent directory")?;
    fs::create_dir_all(parent)?;
    Ok(parent.canonicalize()?.join(name))
}

fn ensure_regular_destination(path: &Path, force: bool) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(meta) => {
            if meta.file_type().is_symlink() || !meta.is_file() {
                return Err(format!(
                    "output must not be a symlink or a directory: {}",
                    path.display()
                )
                .into());
            }
            if !force {
                return Err(
                    format!("output already exists (use --force): {}", path.display()).into(),
                );
            }
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err.into()),
    }
    Ok(())
}

pub struct OutputTargets {
    pub transcript: PathBuf,
    pub audio: Option<PathBuf>,
    pub force: bool,
}

impl OutputTargets {
    pub fn prepare(
        transcript: &Path,
        audio: Option<&Path>,
        force: bool,
        protected: &[&Path],
    ) -> Result<Self> {
        let transcript = normalized_target(transcript)?;
        let audio = audio.map(normalized_target).transpose()?;
        if audio.as_deref() == Some(transcript.as_path()) {
            return Err("transcript and audio outputs refer to the same path".into());
        }
        for target in std::iter::once(transcript.as_path()).chain(audio.as_deref()) {
            for protected_file in protected {
                if protected_file
                    .canonicalize()
                    .is_ok_and(|real| real == target)
                {
                    return Err("an output would overwrite a protected input/model".into());
                }
            }
            ensure_regular_destination(target, force)?;
        }
        Ok(Self {
            transcript,
            audio,
            force,
        })
    }
}

fn stage_bytes(target: &Path, bytes: &[u8]) -> Result<NamedTempFile> {
    let parent = target.parent().ok_or("output has no parent")?;
    let mut temporary = NamedTempFile::new_in(parent)?;
    temporary.write_all(bytes)?;
    temporary.as_file().sync_all()?;
    Ok(temporary)
}

fn stage_file(target: &Path, source: &Path) -> Result<NamedTempFile> {
    let parent = target.parent().ok_or("output has no parent")?;
    let mut temporary = NamedTempFile::new_in(parent)?;
    io::copy(&mut fs::File::open(source)?, temporary.as_file_mut())?;
    temporary.as_file().sync_all()?;
    Ok(temporary)
}

fn publish(temp: NamedTempFile, target: &Path, force: bool) -> Result<()> {
    // Recheck the ordinary no-overwrite case as late as practical.
    // No-overwrite uses OS-level no-clobber persist, not a check-then-open race.
    if force {
        ensure_regular_destination(target, true)?;
        temp.persist(target)?;
    } else {
        temp.persist_noclobber(target)?;
    }
    Ok(())
}

pub fn write_outputs(rendered: &[u8], wav: &Path, targets: &OutputTargets) -> Result<()> {
    // Stage *both* before publishing either: errors during copy leave old results intact.
    let staged_transcript = stage_bytes(&targets.transcript, rendered)?;
    let staged_audio = targets
        .audio
        .as_ref()
        .map(|p| stage_file(p, wav))
        .transpose()?;
    // Publishing two files cannot be a filesystem-wide atomic transaction.
    publish(staged_transcript, &targets.transcript, targets.force)?;
    if let (Some(audio_target), Some(temp_audio)) = (&targets.audio, staged_audio) {
        publish(temp_audio, audio_target, targets.force)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn does_not_overwrite_without_force() {
        let temp = tempfile::tempdir().unwrap();
        let dest = temp.path().join("out.txt");
        fs::write(&dest, "old").unwrap();
        assert!(OutputTargets::prepare(&dest, None, false, &[]).is_err());
        assert_eq!(fs::read_to_string(dest).unwrap(), "old");
    }

    #[test]
    fn hard_link_alias_is_not_truncated() {
        let temp = tempfile::tempdir().unwrap();
        let video = temp.path().join("video.mp4");
        let out = temp.path().join("out.txt");
        fs::write(&video, "ORIGINAL").unwrap();
        fs::hard_link(&video, &out).unwrap();
        let targets = OutputTargets::prepare(&out, None, true, &[&video]).unwrap();
        write_outputs(b"REPLACEMENT", &video, &targets).unwrap();
        assert_eq!(fs::read_to_string(video).unwrap(), "ORIGINAL");
        assert_eq!(fs::read_to_string(out).unwrap(), "REPLACEMENT");
    }

    #[test]
    fn output_alias_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let dir = temp.path().join("dir");
        fs::create_dir(&dir).unwrap();
        let a = temp.path().join("dir/../same.txt");
        let b = temp.path().join("same.txt");
        assert!(OutputTargets::prepare(&a, Some(&b), true, &[]).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_target_rejected_even_with_force() {
        use std::os::unix::fs::symlink;
        let temp = tempfile::tempdir().unwrap();
        let sentinel = temp.path().join("sentinel.txt");
        let link = temp.path().join("link.txt");
        fs::write(&sentinel, "keep").unwrap();
        symlink(&sentinel, &link).unwrap();
        assert!(OutputTargets::prepare(&link, None, true, &[]).is_err());
        assert_eq!(fs::read_to_string(sentinel).unwrap(), "keep");
    }

    #[test]
    fn temporary_staging_failure_does_not_modify_existing_output() {
        let temp = tempfile::tempdir().unwrap();
        let out = temp.path().join("out.txt");
        fs::write(&out, "old").unwrap();
        assert!(stage_file(&out, &temp.path().join("missing.wav")).is_err());
        assert_eq!(fs::read_to_string(out).unwrap(), "old");
    }
}
