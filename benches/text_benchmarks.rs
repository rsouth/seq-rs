use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sequencer::rendering::text::{measure_string, rgb_to_u32};
use sequencer::theme::Theme;

fn measure_measure_string(c: &mut Criterion) {
    let theme = Theme::default();
    c.bench_function("measure_string single char", |b| {
        b.iter(|| measure_string(black_box(&theme), black_box("A"), black_box(20)))
    });
}

fn measure_measure_string_long(c: &mut Criterion) {
    let theme = Theme::default();
    c.bench_function("measure_string long string", |b| {
        b.iter(|| {
            measure_string(
                black_box(&theme),
                black_box("Example Sequence Diagram"),
                black_box(30),
            )
        })
    });
}

fn measure_rgb_to_u32(c: &mut Criterion) {
    c.bench_function("rgb_to_u32", |b| {
        b.iter(|| rgb_to_u32(black_box(128), black_box(64), black_box(32), black_box(255)))
    });
}

criterion_group!(
    benches,
    measure_measure_string,
    measure_measure_string_long,
    measure_rgb_to_u32,
);
criterion_main!(benches);
