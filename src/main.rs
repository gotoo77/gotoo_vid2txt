use clap::{Args, Parser, Subcommand};
use gotoo_vid2txt::{
    config::{AudioBackend, Config, ExportBackend, Overrides, SourceBackend, TranscriptionBackend},
    domain::{Job, Source},
    pipeline::StageTimings,
    registry::{build, validate_source},
    Result,
};
use std::{path::{Path, PathBuf}, process::{self, Command}};

#[derive(Parser)]
#[command(name = "gotoo-vid2txt", version, about = "Configurable, local-first media transcription")]
struct Cli { #[command(subcommand)] command: Commands }

#[derive(Subcommand)]
enum Commands {
    /// Process a local media file or supported URL.
    Transcribe(TranscribeArgs),
    /// Inspect pipeline selection and requirements, without processing media.
    Doctor(DoctorArgs),
}

#[derive(Args, Default)]
struct BackendArgs {
    /// Config TOML (if omitted, built-in defaults; no implicit config search).
    #[arg(long)] config: Option<PathBuf>,
    /// Acquisition selection: auto, local, yt-dlp.
    #[arg(long, value_enum)] source: Option<SourceBackend>,
    /// Decoder/normalizer selection: ffmpeg or wav-pcm16.
    #[arg(long, value_enum)] audio: Option<AudioBackend>,
    /// Speech-to-text engine (currently whisper-cpp).
    #[arg(long, value_enum)] transcriber: Option<TranscriptionBackend>,
    /// Output renderer (txt or json).
    #[arg(long, alias = "format", value_enum)] exporter: Option<ExportBackend>,
    /// whisper.cpp model file (mandatory for execution).
    #[arg(long)] model: Option<PathBuf>,
    #[arg(long)] yt_dlp_bin: Option<PathBuf>,
    #[arg(long)] ffmpeg_bin: Option<PathBuf>,
    #[arg(long)] whisper_bin: Option<PathBuf>,
}

impl BackendArgs {
    fn load(&self) -> Result<Config> {
        let mut c = Config::load(self.config.as_deref())?;
        c.apply(Overrides {
            source: self.source, audio: self.audio, transcriber: self.transcriber,
            exporter: self.exporter, model: self.model.clone(),
            yt_dlp_bin: self.yt_dlp_bin.clone(), ffmpeg_bin: self.ffmpeg_bin.clone(),
            whisper_bin: self.whisper_bin.clone(),
        });
        Ok(c)
    }
}

#[derive(Args)]
struct DoctorArgs {
    #[command(flatten)] backend: BackendArgs,
    /// Optional input: only inspect the acquisition route needed for this input.
    #[arg(long)] input: Option<String>,
}

#[derive(Args)]
struct TranscribeArgs {
    /// Local media file or HTTP(S) URL.
    input: String,
    #[command(flatten)] backend: BackendArgs,
    #[arg(long)] output: Option<PathBuf>,
    #[arg(long, default_value = "auto")] language: String,
    #[arg(long)] audio_out: Option<PathBuf>,
    #[arg(long)] force: bool,
    /// Print the effective pipeline; never download or create outputs.
    #[arg(long)] dry_run: bool,
}

fn output_name(source: &Source, extension: &str) -> PathBuf {
    let stem = match source {
        Source::Local(path) => path.file_stem().and_then(|s| s.to_str()).unwrap_or("transcription"),
        Source::Url(_) => "transcription",
    };
    PathBuf::from(format!("{stem}.{extension}"))
}

fn print_plan(config: &Config, source: &Source, output: &Path, audio_output: Option<&Path>) -> Result<()> {
    let p = &config.profile.default;
    let acquisition = validate_source(p.source, source)?;
    println!("Acquisition : {acquisition}");
    println!("Audio       : {:?}", p.audio);
    println!("Transcriber : {:?}", p.transcriber);
    println!("Exporter    : {:?}", p.exporter);
    println!("Transcript  : {}", output.display());
    if let Some(audio) = audio_output { println!("Audio output: {}", audio.display()); }
    println!("Note: no fallback, model download, or network access is performed by dry-run.");
    Ok(())
}

fn check_program(program: &Path, flag: &str, name: &str) -> Result<()> {
    let result = Command::new(program).arg(flag).output()?;
    if !result.status.success() && result.stdout.is_empty() && result.stderr.is_empty() {
        return Err(format!("{name}: command did not respond successfully").into());
    }
    println!("[responds] {name}: {}", program.display());
    Ok(())
}

fn doctor(args: DoctorArgs) -> Result<()> {
    let config = args.backend.load()?;
    let p = &config.profile.default;
    let uses_url = match args.input.as_deref() {
        Some(v) => validate_source(p.source, &Source::parse(v))? == "yt-dlp",
        None => p.source != SourceBackend::Local,
    };
    if uses_url { check_program(&config.backends.yt_dlp.binary, "--version", "yt-dlp")?; }
    if p.audio == AudioBackend::Ffmpeg {
        check_program(&config.backends.ffmpeg.binary, "-version", "ffmpeg")?;
    } else {
        println!("[native] wav-pcm16 decoder; accepts strictly canonical 16k mono PCM16 WAV");
    }
    match p.transcriber {
        TranscriptionBackend::WhisperCpp => {
            let model = config.backends.whisper_cpp.model.as_ref()
                .ok_or("missing whisper_cpp model (provide --model or TOML configuration)")?;
            if !model.is_file() { return Err(format!("model not found: {}", model.display()).into()); }
            check_program(config.backends.whisper_cpp.binary.as_deref()
                .unwrap_or_else(|| Path::new("whisper-cli")), "--help", "whisper-cli")?;
        }
    }
    println!("[selected] exporter {:?}", p.exporter);
    println!("Doctor checks only program responsiveness and model presence, not inference accuracy.");
    Ok(())
}

fn transcribe(args: TranscribeArgs) -> Result<()> {
    let config = args.backend.load()?;
    let source = Source::parse(&args.input);
    let p = &config.profile.default;
    validate_source(p.source, &source)?;
    let output = args.output.unwrap_or_else(|| output_name(&source, p.exporter.extension()));
    if args.dry_run { return print_plan(&config, &source, &output, args.audio_out.as_deref()); }
    let job = Job {
        source, language: args.language, output, audio_output: args.audio_out,
        force: args.force,
    };
    let pipeline = build(&config)?;
    let report = pipeline.run(&job)?;
    let StageTimings { acquisition, decoding, transcription, publication, total } = report.timings;
    eprintln!(
        "Completed: {} characters; acquisition={acquisition:?}, audio={decoding:?}, inference={transcription:?}, export={publication:?}, total={total:?}",
        report.transcript.text.chars().count()
    );
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Transcribe(args) => transcribe(args),
        Commands::Doctor(args) => doctor(args),
    };
    if let Err(err) = result { eprintln!("error: {err}"); process::exit(1); }
}
