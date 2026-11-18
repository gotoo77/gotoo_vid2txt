use super::Acquisition;
use crate::{domain::Source, Result};
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::Command,
};

pub struct YtDlp {
    pub executable: OsString,
}

impl YtDlp {
    fn command(&self, url: &str, scratch: &Path) -> Command {
        let template = scratch.join("source.%(ext)s");
        let mut cmd = Command::new(&self.executable);
        cmd.arg("--ignore-config")
            .arg("--no-playlist")
            .arg("--no-progress")
            .arg("--quiet")
            .arg("--format")
            .arg("bestaudio/best")
            .arg("--output")
            .arg(template)
            .arg("--print")
            .arg("after_move:filepath")
            .arg("--")
            .arg(url);
        cmd
    }
}

impl Acquisition for YtDlp {
    fn acquire(&self, source: &Source, scratch: &Path) -> Result<PathBuf> {
        let Source::Url(url) = source else {
            return Err("yt-dlp adapter received a local path".into());
        };
        let output = self.command(url, scratch).output()?;
        if !output.status.success() {
            // Avoid leaking credentials embedded in URLs via raw tool diagnostics.
            return Err(format!("yt-dlp acquisition failed ({})", output.status).into());
        }
        let stdout = String::from_utf8(output.stdout)?;
        let last = stdout
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .ok_or("yt-dlp returned no output file")?;
        let path = PathBuf::from(last.trim());
        let path = path.canonicalize()?;
        if !path.is_file() || !path.starts_with(scratch.canonicalize()?) {
            return Err("yt-dlp returned a file outside the private workspace".into());
        }
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn youtube_url_is_passed_as_one_argument_and_configs_disabled() {
        let url = "https://example.org/a?v=a&x=$(echo hi)";
        let cmd = YtDlp {
            executable: "yt-dlp".into(),
        }
        .command(url, Path::new("/tmp/test"));
        let args: Vec<_> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().to_string())
            .collect();
        assert!(args.contains(&"--ignore-config".into()));
        assert!(args.contains(&"--no-playlist".into()));
        assert_eq!(args[args.len() - 2], "--");
        assert_eq!(args.last().unwrap(), url);
    }
}
