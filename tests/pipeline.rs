use gotoo_vid2txt::{
    acquisition::Acquisition,
    audio::AudioDecoder,
    domain::{Job, Source, Transcript},
    export::{Exporter, Json},
    pipeline::Pipeline,
    transcription::Transcriber,
    Result,
};
use std::{fs, path::{Path, PathBuf}, sync::{Arc, Mutex}};

#[derive(Default)]
struct Events(Arc<Mutex<Vec<&'static str>>>);
impl Events {
    fn record(&self, event: &'static str) { self.0.lock().unwrap().push(event); }
    fn values(&self) -> Vec<&'static str> { self.0.lock().unwrap().clone() }
}
impl Clone for Events {
    fn clone(&self) -> Self { Self(Arc::clone(&self.0)) }
}

struct FakeAcquisition(Events);
impl Acquisition for FakeAcquisition {
    fn acquire(&self, _: &Source, scratch: &Path) -> Result<PathBuf> {
        self.0.record("acquire");
        let path = scratch.join("source.bin");
        fs::write(&path, b"media")?;
        Ok(path)
    }
}
struct FakeDecoder(Events);
impl AudioDecoder for FakeDecoder {
    fn normalize(&self, _: &Path, destination: &Path) -> Result<()> {
        self.0.record("decode");
        fs::write(destination, b"wav")?;
        Ok(())
    }
}
struct FakeTranscriber(Events, bool, PathBuf);
impl Transcriber for FakeTranscriber {
    fn id(&self) -> &'static str { "fake" }
    fn protected_paths(&self) -> Vec<&Path> { vec![self.2.as_path()] }
    fn transcribe(&self, _: &Path, _: &str, _: &Path) -> Result<Transcript> {
        self.0.record("transcribe");
        if self.1 { return Err("injected inference failure".into()); }
        Ok(Transcript::new("fake", Some("fr".into()), "bonjour\n".into()))
    }
}

fn job(root: &Path) -> Job {
    Job {
        source: Source::Url("https://example.org/video".into()),
        language: "fr".into(),
        output: root.join("result.json"),
        audio_output: Some(root.join("audio.wav")),
        force: false,
    }
}

#[test]
fn fake_pipeline_validates_stage_order_json_and_audio() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("model.bin"), "model").unwrap();
    let events = Events::default();
    let pipeline = Pipeline {
        acquisition: FakeAcquisition(events.clone()),
        decoder: FakeDecoder(events.clone()),
        transcriber: FakeTranscriber(events.clone(), false, dir.path().join("model.bin")),
        exporter: Json,
    };
    let report = pipeline.run(&job(dir.path())).unwrap();
    assert_eq!(events.values(), ["acquire", "decode", "transcribe"]);
    assert_eq!(report.transcript.text, "bonjour\n");
    let result: serde_json::Value = serde_json::from_slice(&fs::read(dir.path().join("result.json")).unwrap()).unwrap();
    assert_eq!(result["schema_version"], 1);
    assert_eq!(fs::read(dir.path().join("audio.wav")).unwrap(), b"wav");
}

#[test]
fn transcription_failure_leaves_existing_outputs_intact() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("model.bin"), "model").unwrap();
    fs::write(dir.path().join("result.json"), "old").unwrap();
    fs::write(dir.path().join("audio.wav"), "oldwav").unwrap();
    let events = Events::default();
    let pipeline = Pipeline {
        acquisition: FakeAcquisition(events.clone()),
        decoder: FakeDecoder(events.clone()),
        transcriber: FakeTranscriber(events, true, dir.path().join("model.bin")),
        exporter: Json,
    };
    let mut request = job(dir.path());
    request.force = true;
    assert!(pipeline.run(&request).is_err());
    assert_eq!(fs::read_to_string(dir.path().join("result.json")).unwrap(), "old");
    assert_eq!(fs::read_to_string(dir.path().join("audio.wav")).unwrap(), "oldwav");
}

#[test]
fn preflight_rejects_collision_before_acquisition() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("model.bin"), "model").unwrap();
    let events = Events::default();
    let pipeline = Pipeline {
        acquisition: FakeAcquisition(events.clone()),
        decoder: FakeDecoder(events.clone()),
        transcriber: FakeTranscriber(events.clone(), false, dir.path().join("model.bin")),
        exporter: Json,
    };
    let mut request = job(dir.path());
    request.output = dir.path().join("model.bin");
    request.force = true;
    assert!(pipeline.run(&request).is_err());
    assert!(events.values().is_empty());
    assert_eq!(fs::read_to_string(dir.path().join("model.bin")).unwrap(), "model");
}


struct AlternativeTranscriber;
impl Transcriber for AlternativeTranscriber {
    fn id(&self) -> &'static str { "other-engine" }
    fn transcribe(&self, _: &Path, _: &str, _: &Path) -> Result<Transcript> {
        Ok(Transcript::new("other-engine", None, "different".into()))
    }
}

#[test]
fn another_backend_can_replace_whisper_without_changing_pipeline() {
    let dir = tempfile::tempdir().unwrap();
    let events = Events::default();
    let pipeline = Pipeline {
        acquisition: FakeAcquisition(events.clone()),
        decoder: FakeDecoder(events),
        transcriber: AlternativeTranscriber,
        exporter: Json,
    };
    let mut request = job(dir.path());
    request.audio_output = None;
    let report = pipeline.run(&request).unwrap();
    assert_eq!(report.transcript.backend, "other-engine");
    assert_eq!(report.transcript.text, "different");
}

struct FailingAcquisition;
impl Acquisition for FailingAcquisition {
    fn acquire(&self, _: &Source, _: &Path) -> Result<PathBuf> {
        Err("acquisition error injected".into())
    }
}

#[test]
fn acquisition_failure_does_not_publish_anything() {
    let dir = tempfile::tempdir().unwrap();
    let events = Events::default();
    let pipeline = Pipeline {
        acquisition: FailingAcquisition,
        decoder: FakeDecoder(events.clone()),
        transcriber: FakeTranscriber(events.clone(), false, dir.path().join("model.bin")),
        exporter: Json,
    };
    fs::write(dir.path().join("model.bin"), "model").unwrap();
    let request = job(dir.path());
    assert!(pipeline.run(&request).is_err());
    assert!(!request.output.exists());
    assert!(!request.audio_output.unwrap().exists());
    assert_eq!(events.values(), Vec::<&'static str>::new());
}

#[test]
fn backend_mismatch_is_rejected_before_publication() {
    struct BadBackend;
    impl Transcriber for BadBackend {
        fn id(&self) -> &'static str { "declared" }
        fn transcribe(&self, _: &Path, _: &str, _: &Path) -> Result<Transcript> {
            Ok(Transcript::new("different", None, "invalid".into()))
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let events = Events::default();
    let pipeline = Pipeline {
        acquisition: FakeAcquisition(events.clone()),
        decoder: FakeDecoder(events),
        transcriber: BadBackend,
        exporter: Json,
    };
    let mut request = job(dir.path());
    request.audio_output = None;
    assert!(pipeline.run(&request).is_err());
    assert!(!request.output.exists());
}

struct CustomExporter;
impl Exporter for CustomExporter {
    fn id(&self) -> &'static str { "custom" }
    fn extension(&self) -> &'static str { "custom" }
    fn render(&self, transcript: &Transcript) -> Result<Vec<u8>> {
        Ok(format!("backend={}\n", transcript.backend).into_bytes())
    }
}

#[test]
fn exporter_can_be_substituted_without_pipeline_changes() {
    let dir = tempfile::tempdir().unwrap();
    let events = Events::default();
    let pipeline = Pipeline {
        acquisition: FakeAcquisition(events.clone()),
        decoder: FakeDecoder(events),
        transcriber: AlternativeTranscriber,
        exporter: CustomExporter,
    };
    let mut request = job(dir.path());
    request.audio_output = None;
    pipeline.run(&request).unwrap();
    assert_eq!(fs::read_to_string(request.output).unwrap(), "backend=other-engine\n");
}

struct FailingExporter;
impl Exporter for FailingExporter {
    fn id(&self) -> &'static str { "fail" }
    fn extension(&self) -> &'static str { "txt" }
    fn render(&self, _: &Transcript) -> Result<Vec<u8>> { Err("injected export failure".into()) }
}

#[test]
fn export_failure_preserves_preexisting_outputs() {
    let dir = tempfile::tempdir().unwrap();
    let events = Events::default();
    let pipeline = Pipeline {
        acquisition: FakeAcquisition(events.clone()),
        decoder: FakeDecoder(events),
        transcriber: AlternativeTranscriber,
        exporter: FailingExporter,
    };
    let mut request = job(dir.path());
    fs::write(&request.output, "old").unwrap();
    fs::write(request.audio_output.as_ref().unwrap(), "oldwav").unwrap();
    request.force = true;
    assert!(pipeline.run(&request).is_err());
    assert_eq!(fs::read_to_string(request.output).unwrap(), "old");
    assert_eq!(fs::read_to_string(request.audio_output.unwrap()).unwrap(), "oldwav");
}
