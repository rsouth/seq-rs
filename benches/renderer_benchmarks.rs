use criterion::{black_box, criterion_group, criterion_main, Criterion};
use raqote::AntialiasMode;
use sequencer::rendering::*;

fn measure_text_single_char(c: &mut Criterion) {
    let font = get_system_font();
    // let parser = measure_text()
    c.bench_function("measure_text single", |b| {
        b.iter(|| measure_text(&font, 20., "A", AntialiasMode::None))
    });
}

// fn interaction_parser_benchmark_multiline(c: &mut Criterion) {
//     let parser = InteractionParser::default();
//     c.bench_function("interaction_parser multiline", |b| {
//         b.iter(|| parser.parse_interactions(black_box(get_text().lines())))
//     });
// }

criterion_group!(benches, measure_text_single_char);
criterion_main!(benches);
