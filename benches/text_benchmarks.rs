use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sequencer::parser::{Interaction, Participant};
use sequencer::render_context::RenderingContext;
use sequencer::text::{measure_strings, measure_text};

fn measure_measure_strings(c: &mut Criterion) {
    let interaction_set = vec![
        Interaction {
            from_participant: Participant {
                name: "One".to_string(),
            },
            to_participant: Participant {
                name: "Two".to_string(),
            },
            message: None,
            order: 0,
        },
        Interaction {
            from_participant: Participant {
                name: "Two".to_string(),
            },
            to_participant: Participant {
                name: "Three".to_string(),
            },
            message: None,
            order: 0,
        },
    ];

    let font = RenderingContext::get_system_font("Arial");

    c.bench_function("measure_measure_strings", |b| {
        b.iter(|| {
            measure_strings(
                black_box(&font),
                black_box(40.),
                black_box(&interaction_set),
            )
        })
    });
}

fn measure_measure_text(c: &mut Criterion) {
    let font = RenderingContext::get_system_font("Arial");
    let text = "Test Text";

    c.bench_function("measure_text", |b| {
        b.iter(|| measure_text(black_box(&font), black_box(40.), black_box(text)));
    });
}

criterion_group!(benches, measure_measure_strings, measure_measure_text);
criterion_main!(benches);
