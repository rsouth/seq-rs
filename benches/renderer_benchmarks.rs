use criterion::{black_box, criterion_group, criterion_main, Criterion};
use sequencer::parser::{Interaction, InteractionSet, Participant};
use sequencer::rendering::*;
use sequencer::text::measure_text;
use sequencer::Diagram;

fn measure_text_single_char(c: &mut Criterion) {
    let font = RenderingContext::get_system_font("Arial");
    c.bench_function("measure_text single", |b| {
        b.iter(|| measure_text(&font, 20., "A"))
    });
}

fn measure_calculate_diagram_height(c: &mut Criterion) {
    let p: InteractionSet = vec![Interaction {
        from_participant: Participant {
            name: "One".to_string(),
        },
        to_participant: Participant {
            name: "Two".to_string(),
        },
        message: None,
        order: 0,
    }];
    let diagram: Diagram = Diagram { interaction_set: p };
    c.bench_function("calculate_diagram_height", |b| {
        b.iter(|| diagram.calculate_diagram_height())
    });
}

fn measure_calculate_diagram_width(c: &mut Criterion) {
    let p: InteractionSet = vec![Interaction {
        from_participant: Participant {
            name: "One".to_string(),
        },
        to_participant: Participant {
            name: "Two".to_string(),
        },
        message: None,
        order: 0,
    }];
    let diagram: Diagram = Diagram { interaction_set: p };
    let font = &RenderingContext::get_system_font("Arial");
    c.bench_function("calculate_diagram_width", |b| {
        b.iter(|| diagram.calculate_diagram_width(black_box(font), black_box(40.)))
    });
}

criterion_group!(
    benches,
    measure_text_single_char,
    measure_calculate_diagram_height,
    measure_calculate_diagram_width
);
criterion_main!(benches);
