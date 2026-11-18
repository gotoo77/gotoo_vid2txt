use criterion::{criterion_group, criterion_main, Criterion};
use gotoo_vid2txt::config::Config;
use std::hint::black_box;

fn benchmark_toml_config(c: &mut Criterion) {
    let document = r#"
        [profile.default]
        source = "local"
        audio = "wav_pcm16"
        transcriber = "whisper_cpp"
        exporter = "json"
        [backends.whisper_cpp]
        model = "models/base.bin"
    "#;
    c.bench_function("parse_config_toml", |b| {
        b.iter(|| toml::from_str::<Config>(black_box(document)).unwrap())
    });
}
criterion_group!(benches, benchmark_toml_config);
criterion_main!(benches);
