use criterion::{criterion_group, criterion_main, Criterion};
use gotoo_vid2txt::domain::{Segment, Transcript};
use std::hint::black_box;

fn bench_json_serialization(c: &mut Criterion) {
    let mut transcript = Transcript::new("fake", Some("fr".into()), "sample".repeat(1000));
    for i in 0..500 {
        transcript.segments.push(Segment { start_ms: i * 1000, end_ms: (i + 1) * 1000, text: "hello world".to_owned() });
    }
    c.bench_function("serialize_500_segments_json", |b| {
        b.iter(|| serde_json::to_vec(black_box(&transcript)).unwrap())
    });
}

criterion_group!(benches, bench_json_serialization);
criterion_main!(benches);
