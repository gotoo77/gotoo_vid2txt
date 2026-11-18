//! Prefer accessible YouTube captions before running any speech-to-text engine.
use crate::{domain::{Segment, Transcript}, Result};
use serde_json::Value;
use std::{fs, path::Path, process::Command};

pub fn is_youtube_url(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://").or_else(|| url.strip_prefix("http://")) else {
        return false;
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("").to_ascii_lowercase();
    matches!(host.as_str(), "youtube.com" | "www.youtube.com" | "m.youtube.com" | "youtu.be")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind { Manual, Automatic }

struct Track { kind: Kind, code: String }

fn select_track(info: &Value, requested: &str) -> Result<Option<Track>> {
    let original = info.get("language").and_then(Value::as_str);
    let requested = if requested == "auto" {
        let Some(language) = original else { return Ok(None); };
        language
    } else {
        requested
    };
    if requested.is_empty() || !requested.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("invalid caption language".into());
    }
    let base = requested.split('-').next().unwrap_or(requested);
    let native = original.is_some_and(|l| l == base || l.starts_with(&format!("{base}-")));
    for kind in [Kind::Manual, Kind::Automatic] {
        let key = if kind == Kind::Manual { "subtitles" } else { "automatic_captions" };
        let Some(tracks) = info.get(key).and_then(Value::as_object) else { continue; };
        let mut candidates = Vec::new();
        if kind == Kind::Automatic && native { candidates.push(format!("{base}-orig")); }
        candidates.push(requested.to_owned());
        candidates.push(base.to_owned());
        if kind == Kind::Manual && native { candidates.push(format!("{base}-orig")); }
        for name in tracks.keys() {
            if name.starts_with(&format!("{base}-"))
                && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
            {
                candidates.push(name.clone());
            }
        }
        for code in candidates {
            let Some(formats) = tracks.get(&code).and_then(Value::as_array) else { continue; };
            if formats.iter().any(|v| v.get("ext").and_then(Value::as_str) == Some("json3")) {
                return Ok(Some(Track { kind, code }));
            }
            return Err(format!("captions found in {code}, but json3 format is unavailable").into());
        }
    }
    Ok(None)
}

fn parse_json3(bytes: &[u8], kind: Kind, language: &str) -> Result<Transcript> {
    let json: Value = serde_json::from_slice(bytes)?;
    let events = json.get("events").and_then(Value::as_array)
        .ok_or("invalid YouTube captions: missing events")?;
    let mut pieces = Vec::<Segment>::new();
    let mut previous = String::new();
    for event in events {
        let Some(segs) = event.get("segs").and_then(Value::as_array) else { continue; };
        let raw = segs.iter().filter_map(|seg| seg.get("utf8").and_then(Value::as_str))
            .collect::<String>();
        let line = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        if line.is_empty() || line == previous { continue; }
        let remainder = line.strip_prefix(&previous)
            .filter(|s| s.starts_with(' ')).unwrap_or(&line);
        let text = remainder.trim().to_owned();
        previous = line;
        if text.is_empty() { continue; }
        let start = event.get("tStartMs").and_then(Value::as_u64).unwrap_or(0);
        let duration = event.get("dDurationMs").and_then(Value::as_u64).unwrap_or(0);
        pieces.push(Segment {
            start_ms: start,
            end_ms: start.saturating_add(duration),
            text,
        });
    }
    if pieces.is_empty() { return Err("YouTube captions contain no usable text".into()); }
    let combined = pieces.iter().map(|v| v.text.as_str()).collect::<Vec<_>>().join(" ");
    let name = if kind == Kind::Manual { "youtube-manual-captions" } else { "youtube-auto-captions" };
    let mut transcript = Transcript::new(name, Some(language.to_owned()), combined);
    transcript.segments = pieces;
    if kind == Kind::Automatic {
        transcript.warnings.push("YouTube auto captions may contain recognition errors".to_owned());
    }
    Ok(transcript)
}

pub fn try_transcript(executable: &Path, url: &str, requested_language: &str) -> Result<Option<Transcript>> {
    let result = Command::new(executable)
        .args(["--ignore-config", "--no-playlist", "--quiet", "--skip-download", "--dump-single-json", "--"])
        .arg(url).output()?;
    if !result.status.success() {
        // Do not leak signed media URLs or credentials from external tool diagnostics.
        return Err(format!("yt-dlp caption metadata failed ({})", result.status).into());
    }
    let info: Value = serde_json::from_slice(&result.stdout)?;
    let Some(track) = select_track(&info, requested_language)? else { return Ok(None); };

    let mut builder = tempfile::Builder::new();
    builder.prefix("gotoo-vid2txt-captions-");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        builder.permissions(std::fs::Permissions::from_mode(0o700));
    }
    let workspace = builder.tempdir()?;
    let status = Command::new(executable)
        .args(["--ignore-config", "--no-playlist", "--quiet", "--no-progress",
               "--skip-download", "--sub-format", "json3", "--sub-langs"])
        .arg(&track.code)
        .arg(if track.kind == Kind::Manual { "--write-subs" } else { "--write-auto-subs" })
        .arg("--output")
        .arg(workspace.path().join("captions.%(ext)s"))
        .arg("--").arg(url).status()?;
    if !status.success() {
        return Err(format!("yt-dlp caption download failed ({status})").into());
    }
    let bytes = fs::read(workspace.path().join(format!("captions.{}.json3", track.code)))?;
    let base = track.code.split('-').next().unwrap_or(&track.code);
    Ok(Some(parse_json3(&bytes, track.kind, base)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validates_host_not_just_substring() {
        assert!(is_youtube_url("https://www.youtube.com/shorts/example"));
        assert!(is_youtube_url("https://youtu.be/example"));
        assert!(!is_youtube_url("https://youtube.com.evil.tld/shorts/example"));
    }

    #[test]
    fn manual_captions_have_priority() {
        let metadata = json!({
            "language":"fr-FR",
            "subtitles":{"fr":[{"ext":"json3"}]},
            "automatic_captions":{"fr-orig":[{"ext":"json3"}]}
        });
        let selection = select_track(&metadata, "auto").unwrap().unwrap();
        assert_eq!(selection.kind, Kind::Manual);
        assert_eq!(selection.code, "fr");
    }

    #[test]
    fn prefers_original_automatic_track() {
        let metadata = json!({"language":"fr-FR","subtitles":{},
            "automatic_captions":{"fr":[{"ext":"json3"}],"fr-orig":[{"ext":"json3"}]}});
        let selection = select_track(&metadata, "fr").unwrap().unwrap();
        assert_eq!(selection.code, "fr-orig");
        assert_eq!(selection.kind, Kind::Automatic);
    }

    #[test]
    fn only_absence_signals_fallback() {
        let empty = json!({"language":"fr-FR","subtitles":{},"automatic_captions":{}});
        assert!(select_track(&empty, "auto").unwrap().is_none());
        let unsupported = json!({"language":"fr-FR","subtitles":{"fr":[{"ext":"vtt"}]}});
        assert!(select_track(&unsupported, "fr").is_err());
    }

    #[test]
    fn parses_deduplicated_json3_and_provenance() {
        let source = br#"{"events":[
          {"tStartMs":0,"dDurationMs":100,"segs":[{"utf8":"Bonsoir"}]},
          {"tStartMs":100,"dDurationMs":100,"segs":[{"utf8":"Bonsoir tout le monde"}]},
          {"tStartMs":200,"dDurationMs":100,"segs":[{"utf8":"Bonsoir tout le monde"}]},
          {"tStartMs":300,"dDurationMs":100,"segs":[{"utf8":"Salut"}]}
        ]}"#;
        let parsed = parse_json3(source, Kind::Automatic, "fr").unwrap();
        assert_eq!(parsed.text, "Bonsoir tout le monde Salut");
        assert_eq!(parsed.segments.len(), 3);
        assert_eq!(parsed.backend, "youtube-auto-captions");
    }

    #[test]
    fn invalid_caption_payload_fails_closed() {
        assert!(parse_json3(br#"{}"#, Kind::Manual, "fr").is_err());
        assert!(parse_json3(br#"{"events":[]}"#, Kind::Manual, "fr").is_err());
    }
}
