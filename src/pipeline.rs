//! Small orchestration layer: business stages, timing and error propagation.
use crate::{
    acquisition::Acquisition,
    audio::AudioDecoder,
    domain::{Job, Source, Transcript},
    export::Exporter,
    output::{write_outputs, OutputTargets},
    transcription::Transcriber,
    Result,
};
use std::{path::Path, time::{Duration, Instant}};

#[derive(Debug)]
pub struct StageTimings {
    pub acquisition: Duration,
    pub decoding: Duration,
    pub transcription: Duration,
    pub publication: Duration,
    pub total: Duration,
}

#[derive(Debug)]
pub struct RunReport {
    pub transcript: Transcript,
    pub timings: StageTimings,
}

pub struct Pipeline<A, D, T, E> {
    pub acquisition: A,
    pub decoder: D,
    pub transcriber: T,
    pub exporter: E,
}

impl<A: Acquisition, D: AudioDecoder, T: Transcriber, E: Exporter> Pipeline<A, D, T, E> {
    pub fn run(&self, job: &Job) -> Result<RunReport> {
        if job.language.trim().is_empty() || job.language.starts_with('-') {
            return Err("invalid language".into());
        }
        self.transcriber.preflight()?;
        let source_path = match &job.source {
            Source::Local(file) => Some(file.as_path()),
            Source::Url(_) => None,
        };
        let mut protected: Vec<&Path> = self.transcriber.protected_paths();
        if let Some(source) = source_path {
            protected.push(source);
        }
        // Preflight: refuse unsafe outputs before triggering any external command.
        let outputs = OutputTargets::prepare(
            &job.output,
            job.audio_output.as_deref(),
            job.force,
            &protected,
        )?;

        let started = Instant::now();
        let scratch = tempfile::Builder::new().prefix("gotoo-vid2txt-").tempdir()?;
        let start = Instant::now();
        let input = self.acquisition.acquire(&job.source, scratch.path())?;
        let acquisition = start.elapsed();

        let wav = scratch.path().join("normalized.wav");
        let start = Instant::now();
        self.decoder.normalize(&input, &wav)?;
        let decoding = start.elapsed();

        let start = Instant::now();
        let transcript = self.transcriber.transcribe(
            &wav, &job.language, scratch.path(),
        )?;
        let transcription = start.elapsed();
        if transcript.schema_version != 1 || transcript.backend != self.transcriber.id() {
            return Err("transcriber returned an incompatible result".into());
        }
        let start = Instant::now();
        let bytes = self.exporter.render(&transcript)?;
        write_outputs(&bytes, &wav, &outputs)?;
        let publication = start.elapsed();

        Ok(RunReport {
            transcript,
            timings: StageTimings {
                acquisition,
                decoding,
                transcription,
                publication,
                total: started.elapsed(),
            },
        })
    }
}
